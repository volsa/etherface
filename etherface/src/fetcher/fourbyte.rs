//! Fetcher for <https://www.4byte.directory/>
//!
//! Polls the <https://www.4byte.directory/api/v1/signatures/> and <https://www.4byte.directory/api/v1/event-signatures/>
//! API endpoints every [`FETCHER_POLLING_SLEEP_TIME`] seconds inserting new signatures into the database. 
//! Instead of retrieving all pages from these paginated API endpoints however, the fetcher only retrieves the latest 
//! pages that contain signatures not present in our database. That is fetch one page, check if the page contains any signature
//! already present in our database and if not continue with the next page until the condition no longer is valid in which case
//! sleep before repeating the process starting from page one again.

use crate::fetcher::Fetcher;
use crate::fetcher::FETCHER_POLLING_SLEEP_TIME;
use anyhow::Error;
use chrono::Utc;
use etherface_lib::api::fourbyte::FourbyteClient;
use etherface_lib::database::handler::DatabaseClient;
use etherface_lib::model::MappingSignatureFourbyte;
use etherface_lib::model::SignatureWithMetadata;

#[derive(Debug)]
pub struct FourbyteFetcher;

impl Fetcher for FourbyteFetcher {
    fn start(&self) -> Result<(), Error> {
        let dbc = DatabaseClient::new()?;

        // Check if this the first run and if so retrieve and insert all event / function signatures from 4Byte
        // into our database
        if dbc.mapping_signature_fourbyte().get_events_count() == 0 {
            initial_data_retrieval(&dbc, false)?;
        }

        if dbc.mapping_signature_fourbyte().get_functions_count() == 0 {
            initial_data_retrieval(&dbc, true)?;
        }

        // Main loop; Retrieve one function / event page at a time from 4Byte and insert all signatures from the
        // page that are currently not present in our database. If a signature is present in our database we can
        // safely assume that our database is in sync with the 4Byte signature database and sleep n minutes before
        // re-doing the process. See the following links on how 4Byte pagination works:
        // - https://www.4byte.directory/api/v1/signatures/
        // - https://www.4byte.directory/api/v1/event-signatures/
        loop {
            // Create new client with each iteration because of internal (index) modifications
            let mut fbc = FourbyteClient::new();

            while let Some(signatures) = fbc.page_event_signature()? {
                if insert_signature(&signatures, &dbc) == 0 {
                    break;
                }
            }

            while let Some(signatures) = fbc.page_function_signature()? {
                if insert_signature(&signatures, &dbc) == 0 {
                    break;
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(FETCHER_POLLING_SLEEP_TIME));
        }
    }
}

fn initial_data_retrieval(dbc: &DatabaseClient, function_endpoint: bool) -> Result<(), Error> {
    let mut fbc = FourbyteClient::new();

    println!("Retrieving all 4Byte signatures...");
    let mut signatures = Vec::new();
    match function_endpoint {
        true => {
            while let Some(mut signatures_page) = fbc.page_function_signature()? {
                signatures.append(&mut signatures_page);
            }
        }
        false => {
            while let Some(mut signatures_page) = fbc.page_event_signature()? {
                signatures.append(&mut signatures_page);
            }
        }
    }

    println!("Inserting retrieved signatures...");
    for signature in signatures {
        let inserted_signature = dbc.signature().insert(&signature);
        let mapping = MappingSignatureFourbyte {
            signature_id: inserted_signature.id,
            kind: signature.kind,
            added_at: Utc::now(),
        };

        dbc.mapping_signature_fourbyte().insert(&mapping);
    }

    Ok(())
}

fn insert_signature(signatures: &Vec<SignatureWithMetadata>, dbc: &DatabaseClient) -> usize {
    let mut insert_count = 0;

    for signature in signatures {
        let inserted_signature = dbc.signature().insert(signature);
        let mapping = MappingSignatureFourbyte {
            signature_id: inserted_signature.id,
            kind: signature.kind,
            added_at: Utc::now(),
        };

        match dbc.mapping_signature_fourbyte().get(&mapping) {
            // Signature already exists in our database; we're in sync with 4Byte
            Some(_) => return insert_count,

            // Signature does not exist in our database; insert new signature
            None => {
                dbc.mapping_signature_fourbyte().insert(&mapping);
                insert_count += 1;
            }
        }
    }

    insert_count
}
