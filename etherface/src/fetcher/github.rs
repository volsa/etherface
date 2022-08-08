//! Fetcher for <https://github.com/>
//!
//! Fetcher finding repositories with Solidity code by a combination of using the GitHub Search API as well as
//! focused crawling. This is done with event-threads, where 3 events exist namely [`Event::SearchRepositories`],
//! [`Event::CheckRepositories`] and [`Event::CheckUsers`]. These events are triggered periodically using
//! [`start_background_event`] sending a message with `std::sync:mpsc` to the fetchers main-loop.
//! Within the main-loop either [`GithubCrawler::start_one_crawling_iteration`] is executed or an event if 
//! triggered. The main-loop, using `std::sync:mpsc`, operates in a FIFO manner meaning events may need to wait
//! until one crawling iteration / other currently curring event has successfuly terminated.
//! //! <div align="center">
//!  <img src="" width="250" height="250"> // TODO: Populate URL
//! </div>

use chrono::Date;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use etherface_lib::api::github::GithubClient;
use etherface_lib::database::handler::DatabaseClient;
use etherface_lib::error::Error;
use etherface_lib::model::GithubRepository;
use etherface_lib::model::GithubUser;
use etherface_lib::model::MappingStargazer;
use log::debug;
use log::info;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use super::Fetcher;

#[derive(Debug)]
pub struct GithubFetcher;

impl Fetcher for GithubFetcher {
    fn start(&self) -> Result<(), anyhow::Error> {
        Ok(GithubCrawler::new()?.start()?)
    }
}

#[derive(Clone, Copy, Debug)]
enum Event {
    /// Event to search for Solidity repositories created or updated in a given timerange, where the timerange
    /// is configurable.
    SearchRepositories,

    /// Event to check for Solidity repository updates which were active in the last N days, where N is
    /// configurable.
    CheckRepositories,

    /// Event to check for Solidity repository owner updates which were active in the last N days, where N is
    /// configurable.
    CheckUsers,
}

struct ChannelMessage {
    pub event: Event,
    pub new_event_date: DateTime<Utc>,
}

pub struct GithubCrawler {
    dbc: DatabaseClient,
    ghc: GithubClient,
}

/// The number of users and/or repositories we want to visit per crawling iteration.
/// Choosing a higher number means longer crawling iterations which _may_ set events into a queue until the
/// iteration is done; for example if an iteration takes ~1 hour for N resource visits, then no event can be
/// executed within that timeframe but will instead be queued in a FIFO manner.
const NUM_RESOURCE_VISITS_PER_CRAWLING_ITERATION: usize = 50;

impl GithubCrawler {
    pub fn new() -> Result<Self, Error> {
        Ok(GithubCrawler {
            dbc: DatabaseClient::new()?,
            ghc: GithubClient::new()?,
        })
    }

    pub fn start(&self) -> Result<(), Error> {
        // Check if this is the first ever run and if so fetch all Solidity repositories created between 2015
        // and today's date.
        if self.dbc.github_repository().get_total_count() == 0 {
            for repo in self.search_solidity_repositories_starting_from(Utc.ymd(2015, 1, 1), true)? {
                self.insert_repository_if_not_exists(&repo, false)?;
            }
        }

        let (tx, rx): (Sender<ChannelMessage>, Receiver<ChannelMessage>) = mpsc::channel();
        start_background_event(tx.clone(), Event::SearchRepositories, chrono::Duration::days(1))?;
        start_background_event(tx.clone(), Event::CheckRepositories, chrono::Duration::days(21))?;
        start_background_event(tx, Event::CheckUsers, chrono::Duration::days(21))?;

        // Sleep a few seconds to give the background event schedulers some time to fetch data from the
        // database and issue events if possible
        std::thread::sleep(std::time::Duration::from_secs(5));

        loop {
            match rx.try_recv() {
                Ok(msg) => match msg.event {
                    Event::SearchRepositories => {
                        debug!("Starting SearchRepositories event");
                        let prev_event_date = self.dbc.crawler_metadata().get().last_repository_search.date();

                        debug!("Prev event date: {prev_event_date}");
                        self.insert_recently_created_solidity_repositories(prev_event_date)?;
                        self.upsert_recently_updated_solidity_repositories(prev_event_date)?;

                        // Only set if previous function calls were successful
                        debug!("Prev event date: {}", msg.new_event_date);
                        self.dbc.crawler_metadata().update_last_repository_search_date(msg.new_event_date);
                        debug!("{}", self.dbc.crawler_metadata().get().last_repository_search.date());
                    }

                    Event::CheckRepositories => {
                        debug!("Starting CheckRepositories event");
                        self.find_repository_updates(180)?;

                        // Only set if previous function calls were successful
                        self.dbc.crawler_metadata().update_last_repository_check_date(msg.new_event_date);
                    }

                    Event::CheckUsers => {
                        debug!("Starting CheckUser event");
                        self.find_user_updates(180)?;

                        // Only set if previous commands were successful
                        self.dbc.crawler_metadata().update_last_user_check_date(msg.new_event_date);
                    }
                },

                Err(why) => match why {
                    mpsc::TryRecvError::Empty => self.start_one_crawling_iteration()?,
                    mpsc::TryRecvError::Disconnected => return Err(Error::CrawlerChannelDisconnected),
                },
            }
        }
    }

    /// Starts one crawling iteration which can be summarised as:
    /// Check if there are any unvisited Solidity repository owners (GitHub users)
    ///     Yes => Take the first [`NUM_RESOURCE_VISITS_PER_CRAWLING_ITERATION`] owners from the database and
    ///            retrieve their owned + starred repositories; set them as visited
    ///     No  => Take the first [`NUM_RESOURCE_VISITS_PER_CRAWLING_ITERATION`] unvisited repositories from 
    ///            the database and for each one of them fetch their stargazers; for each fetched stargazer
    ///            retrieve their owner + starred repositories; set them and the repository as visited
    fn start_one_crawling_iteration(&self) -> Result<(), Error> {
        let unvisited_solidity_repository_owners =
            self.dbc.github_user().get_unvisited_solidity_repository_owners_orderd_by_added_at();
        debug!("Starting one crawling iteration");

        match unvisited_solidity_repository_owners.is_empty() {
            false => {
                debug!(
                    "Visiting unvisited solidity repository owners (len: {})",
                    unvisited_solidity_repository_owners.len()
                );
                for owner in unvisited_solidity_repository_owners
                    .iter()
                    .take(NUM_RESOURCE_VISITS_PER_CRAWLING_ITERATION)
                {
                    self.get_and_insert_user_owned_repos(owner.id, true)?;
                    self.get_and_insert_user_starred_repos(owner.id, true)?;

                    self.dbc.github_user().set_visited(owner.id);
                }
            }

            true => {
                let unvisited_repos = self.dbc.github_repository().get_unvisited_ordered_by_added_at();
                debug!("Visiting unvisited solidity repositories (len: {})", unvisited_repos.len());

                if unvisited_repos.is_empty() {
                    panic!(
                        "If you read this message, the crawler is not able to find further repositories;
                        The reason for this is because all Solidity repositories in our database have been
                        visited. There are a couple of ways to bypass this issue:
                        - Issue an event hoping to find new repositories
                        - Sleep until the next event
                        - Widen the crawlers bubble to not only focus on Solidity repositories / owners, but
                          we don't want this as we would get a lot of garbage input
                        - ...
                        We could also use the time to check if our local data needs to be updated to mirror
                        GitHub's state."
                    );
                }

                for repo in unvisited_repos.iter().take(NUM_RESOURCE_VISITS_PER_CRAWLING_ITERATION) {
                    let stargazers = self.get_stargazers_or_set_repository_deleted(repo.id)?;
                    // trace!("Visiting {}", repo.html_url);

                    for stargazer in stargazers {
                        // trace!("On {idx} of {}", stargazers.len());
                        self.dbc.github_user().insert_if_not_exists(&stargazer);

                        self.get_and_insert_user_owned_repos(stargazer.id, true)?;
                        self.get_and_insert_user_starred_repos(stargazer.id, true)?;

                        self.dbc.mapping_stargazer().insert(&MappingStargazer {
                            user_id: stargazer.id,
                            repository_id: repo.id,
                        });

                        self.dbc.github_user().set_visited(stargazer.id);
                    }

                    self.dbc.github_repository().set_visited(repo.id);
                }
            }
        }

        Ok(())
    }
}

/// Helper Functions
impl GithubCrawler {
    fn get_and_insert_user_owned_repos(&self, user_id: i32, crawled: bool) -> Result<(), Error> {
        if let Ok(repos) = self.ghc.user(user_id).repos() {
            for repo in repos {
                self.insert_repository_if_not_exists(&repo, crawled)?;
            }
        }

        Ok(())
    }

    fn get_and_insert_user_starred_repos(&self, user_id: i32, crawled: bool) -> Result<(), Error> {
        if let Ok(repos) = self.ghc.user(user_id).starred() {
            for repo in repos {
                self.insert_repository_if_not_exists(&repo, crawled)?;
                self.dbc.mapping_stargazer().insert(&MappingStargazer {
                    user_id,
                    repository_id: repo.id,
                });
            }
        }

        Ok(())
    }

    fn insert_repository_if_not_exists(&self, entity: &GithubRepository, crawled: bool) -> Result<(), Error> {
        if let Some(repo) = self.dbc.github_repository().get_by_id(entity.id) {
            if repo.is_deleted {
                // Update the deleted status; this can happen if a repository was set to be private rather
                // than deleted and we re-found it within our crawling process
                self.dbc.github_repository().set_undeleted(repo.id);
            }

            return Ok(());
        }

        self.dbc.github_user().insert_if_not_exists(&entity.owner);
        self.dbc.github_repository().insert(entity, 0.0, crawled);

        // Repositories created prior to 2018 are most likely not that interesting because according to our
        // data harvested from GitHub Solidity development started in 2018 and really kicked in in Q3 of 2020
        // As such we simply ignore repositories prior to 2018 in that we save them to the database but don't
        // spend further API calls to check what their languages / Solidity ratio is.
        // For references, from 2015 to 2018 around ~500 repos were created, whereas in 2018 alone ~3000 were
        // created as such we're fine if we lose a few repositories but instead improve crawling speed.
        if entity.created_at.date() <= Utc.ymd(2018, 1, 1) {
            return Ok(());
        }

        // Fetch the Solidity ratio of the given repository
        if let Some(ratio) = self.get_solidity_ratio_or_set_repository_deleted(entity.id)? {
            self.dbc.github_repository().set_ratio(entity.id, ratio);

            // Check if the repository is a fork and if so get a) their parent and b) all other forks
            // Normally we're not too keen in forks, but if someone forked a repository with Solidity code
            // they're a person of interest to us
            if let Some(parent) = &entity.fork_parent {
                // Recursive call, should however end with the first recursion because there's only one
                // true parent (i.e. if a fork forks another fork they'll still point to the same parent)
                self.insert_repository_if_not_exists(parent, true)?;

                // To save some API calls we'll simply assume the ratio to be the same as the parents'
                for fork in self.ghc.repos(parent.id).forks()? {
                    self.dbc.github_user().insert_if_not_exists(&fork.owner);
                    self.dbc.github_repository().insert(&fork, ratio, true);
                }
            }
        }

        Ok(())
    }

    fn search_solidity_repositories_starting_from(
        &self,
        mut from: Date<Utc>,
        query_by_created: bool,
    ) -> Result<Vec<GithubRepository>, Error> {
        let mut repositories = Vec::new();

        let to = Utc::now().date();
        while from <= to {
            match query_by_created {
                true => repositories.append(&mut self.ghc.search().solidity_repos_created_at(from)?),
                false => repositories.append(&mut self.ghc.search().solidity_repos_updated_at(from)?),
            }

            from = from + chrono::Duration::days(1);
        }

        Ok(repositories)
    }

    fn insert_recently_created_solidity_repositories(&self, date: Date<Utc>) -> Result<(), Error> {
        let repos = self.search_solidity_repositories_starting_from(date, true)?;
        println!("About to insert {} repositories", repos.len());

        for repo in repos {
            self.insert_repository_if_not_exists(&repo, false)?;
        }

        Ok(())
    }

    fn upsert_recently_updated_solidity_repositories(&self, date: Date<Utc>) -> Result<(), Error> {
        let repos = self.search_solidity_repositories_starting_from(date, false)?;
        println!("About to upsert {} repos", repos.len());

        for repo in self.search_solidity_repositories_starting_from(date, false)? {
            if self.dbc.github_repository().get_by_id(repo.id).is_none() {
                self.insert_repository_if_not_exists(&repo, false)?;
                continue; // Nothing to do, we inserted the latest version into the database
            }

            // Repository already present in database, update it and re-trigger the scraping process
            if let Some(ratio) = self.get_solidity_ratio_or_set_repository_deleted(repo.id)? {
                println!("Updating {}", repo.html_url);
                self.dbc.github_repository().update(&repo, ratio);
                self.dbc.github_repository().set_scraped_to_null(repo.id);
            }
        }

        Ok(())
    }

    fn find_repository_updates(&self, days: i64) -> Result<(), Error> {
        let sol_repos_active_in_last_n_days =
            self.dbc.github_repository().get_solidity_repos_active_in_last_n_days(days);
        info!("Checking {} repositories for updates", sol_repos_active_in_last_n_days.len());

        for repo_db in sol_repos_active_in_last_n_days {
            match self.ghc.repos(repo_db.id).modified_since(repo_db.updated_at) {
                Ok(repo_gh) => {
                    if let Some(repo_gh) = repo_gh {
                        if repo_gh.pushed_at != repo_db.pushed_at {
                            if let Some(ratio) =
                                self.get_solidity_ratio_or_set_repository_deleted(repo_gh.id)?
                            {
                                self.dbc.github_repository().update(&repo_gh, ratio);
                                self.dbc.github_repository().set_scraped_to_null(repo_gh.id);
                            }
                        }
                    }
                }

                Err(why) => match why {
                    Error::GithubResourceUnavailable(_) => {
                        self.dbc.github_repository().set_deleted(repo_db.id);
                    }

                    _ => return Err(why),
                },
            }
        }

        Ok(())
    }

    fn find_user_updates(&self, days: i64) -> Result<(), Error> {
        let sol_repository_owners_active_in_last_n_days =
            self.dbc.github_user().get_solidity_repository_owners_active_in_last_n_days(days);
        info!(
            "Checking {} Solidity repository owners for updates",
            sol_repository_owners_active_in_last_n_days.len()
        );

        for user_db in sol_repository_owners_active_in_last_n_days {
            match self.ghc.user(user_db.id).get() {
                Ok(user_gh) => {
                    if user_gh.public_repos.unwrap() as i64 != self.dbc.github_user().repo_count(user_gh.id) {
                        for repo in self.ghc.user(user_gh.id).repos()? {
                            self.insert_repository_if_not_exists(&repo, true)?;
                        }
                    }
                }

                Err(why) => match why {
                    Error::GithubResourceUnavailable(_) => {
                        self.dbc.github_user().set_deleted(user_db.id);
                    }

                    _ => return Err(why),
                },
            }
        }

        Ok(())
    }

    #[inline]
    fn get_solidity_ratio_or_set_repository_deleted(&self, repo_id: i32) -> Result<Option<f32>, Error> {
        match self.ghc.repos(repo_id).solidity_ratio() {
            Ok(ratio) => Ok(Some(ratio)),

            Err(why) => match why {
                Error::GithubResourceUnavailable(_) => {
                    self.dbc.github_repository().set_deleted(repo_id);

                    Ok(None)
                }

                _ => Err(why),
            },
        }
    }

    #[inline]
    fn get_stargazers_or_set_repository_deleted(&self, repo_id: i32) -> Result<Vec<GithubUser>, Error> {
        match self.ghc.repos(repo_id).stargazers() {
            Ok(stargazers) => Ok(stargazers),

            Err(why) => match why {
                Error::GithubResourceUnavailable(_) => {
                    self.dbc.github_repository().set_deleted(repo_id);

                    Ok(Vec::with_capacity(0))
                }

                _ => Err(why),
            },
        }
    }
}

fn start_background_event(
    tx: Sender<ChannelMessage>,
    event: Event,
    freq: chrono::Duration,
) -> Result<(), Error> {
    let dbc = DatabaseClient::new()?;
    let last_event_date = match event {
        Event::SearchRepositories => dbc.crawler_metadata().get().last_repository_search,
        Event::CheckRepositories => dbc.crawler_metadata().get().last_repository_check,
        Event::CheckUsers => dbc.crawler_metadata().get().last_user_check,
    };

    std::thread::spawn(move || {
        let delta = Utc::now() - last_event_date;
        if delta < freq {
            // debug!("Sleeping {} minutes before sending {event:#?} event", (freq - delta).num_minutes());
            debug!(
                "Event {event:?} due on {} ({} hours)",
                Utc::now() + (freq - delta),
                (freq - delta).num_hours()
            );
            std::thread::sleep((freq - delta).to_std().unwrap());
        }

        loop {
            debug!("Sending event: {event:#?}");
            tx.send(ChannelMessage {
                event,
                new_event_date: Utc::now(),
            })
            .unwrap();

            std::thread::sleep(freq.to_std().unwrap());
        }
    });
    Ok(())
}
