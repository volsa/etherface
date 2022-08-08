use crate::scraper::Scraper;
use anyhow::Error;
use chrono::Utc;
use etherface_lib::api::github::GithubClient;
use etherface_lib::database::handler::DatabaseClient;
use etherface_lib::model::MappingSignatureGithub;
use etherface_lib::parser;
use log::debug;
use log::error;
use std::process::Command;
use std::process::Stdio;
use std::thread::sleep;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct GithubScraper;

struct File {
    path: String,
    kind: FileKind,
}

enum FileKind {
    Solidity,
    Json,
}

const PATH_CLONE_DIR: &str = "/tmp/etherface";

impl Scraper for GithubScraper {
    fn start(&self) -> Result<(), Error> {
        let ghc = GithubClient::new()?;
        let dbc = DatabaseClient::new()?;

        std::fs::create_dir_all(PATH_CLONE_DIR)?;

        loop {
            println!("Scraping Job-Queue: {}", dbc.github_repository().get_unscraped_with_forks().len());

            for repo in dbc.github_repository().get_unscraped_with_forks() {
                // Repository names within GitHub can start with a dash, which any CLI application such as `git`
                // interprets as an argument. Hence we pre-emptively replace ALL dashes with an underscore because
                // something like `git clone https://github.com/foo/-bar -bar` would result in an error rather
                // than cloning the repository under the name `-bar`. The repository will instead be cloned
                // under the name `_bar` with this solution. Note that we could also just remove the first n `-`
                // characters from the name but names with only dashes are also supported. Instead of doing some
                // fancy magic (a.k.a. supporting edge-cases) we do it the simple and boring way.
                let mut clone_name = repo.name.replace('-', "_");
                clone_name = format!("{PATH_CLONE_DIR}/{}", clone_name.replace('.', "_"));

                let git_clone_command = match Command::new("git")
                    .args([
                        "clone",
                        // Sometimes repositories either get deleted or made private before we have the chance to
                        // clone them; if this happens the default behaviour of git is to ask for a username and
                        // password (in case it's private and you're the owner). Hence we add a `username:password`
                        // to the URL which disables this behaviour such that we are not stuck in that prompt.
                        &repo.html_url.replace("https://github.com", "https://volsa:volsa@github.com"),
                        &clone_name,
                    ])
                    .stderr(Stdio::null()) // Suppress `git clone` output
                    .status()
                {
                    Ok(status) => status,
                    Err(why) => {
                        println!("Failed to clone {}; {why}", repo.html_url);
                        continue;
                    }
                };

                if !git_clone_command.success() {
                    match ghc.repos(repo.id).get() {
                        Ok(_) => {
                            error!("Repository available but failed to clone: {}", repo.html_url);
                            // Set it as scraped and re-try in the next scraping cycle
                            dbc.github_repository().set_scraped(repo.id);
                            continue;
                        }

                        Err(why) => match why {
                            etherface_lib::error::Error::GithubResourceUnavailable(_) => {
                                debug!("Setting {} as deleted", repo.html_url);
                                dbc.github_repository().set_deleted(repo.id);
                                continue;
                            }

                            _ => {
                                // XXX: Never happend so far, hence just log for now; We could set it to scraped
                                //      here though just like in the Ok(_) case
                                error!("Failed to clone; {why}");
                                continue;
                            }
                        },
                    }
                }

                println!("Scraping {}", clone_name);
                for file in get_sol_files(&clone_name) {
                    if let Ok(content) = std::fs::read_to_string(&file.path) {
                        let signatures = match file.kind {
                            FileKind::Solidity => parser::from_sol(&content),
                            FileKind::Json => match parser::from_abi(&content) {
                                Ok(val) => val,
                                Err(_) => continue, // Not a valid JSON ABI file
                            },
                        };

                        for signature in signatures {
                            let signature_db = dbc.signature().insert(&signature);

                            let mapping_entity = MappingSignatureGithub {
                                signature_id: signature_db.id,
                                repository_id: repo.id,
                                kind: signature.kind,
                                visibility: signature.visibility,
                                added_at: Utc::now(),
                            };

                            dbc.mapping_signature_github().insert(&mapping_entity);
                        }
                    }
                }

                dbc.github_repository().set_scraped(repo.id);
                std::fs::remove_dir_all(clone_name)?;
                // sleep(std::time::Duration::from_secs(3));
            }

            // Sleep 5 minutes between each iteration
            println!("Done scraping, sleeping 5 minutes");
            sleep(std::time::Duration::from_secs(5 * 60));
        }
    }
}

/// Returns a list of found Solidity file paths within a directory.
#[inline]
fn get_sol_files(dir_name: &str) -> Vec<File> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir_name).into_iter().filter_map(|x| x.ok()) {
        if let Some(path) = entry.path().to_str() {
            if path.ends_with(".sol") {
                files.push(File {
                    path: path.to_string(),
                    kind: FileKind::Solidity,
                });
            }

            if path.ends_with(".json") || path.ends_with(".abi") {
                files.push(File {
                    path: path.to_string(),
                    kind: FileKind::Json,
                });
            }
        }
    }

    files
}
