//! 4Byte API client.
//!
//! Currently only covers the [`/api/v1/signatures`](https://www.4byte.directory/api/v1/signatures/) and
//! [`/api/v1/event-signatures`](https://www.4byte.directory/api/v1/event-signatures/) endpoints (all we really
//! need).
use crate::error::Error;
use crate::model::SignatureKind;
use crate::model::SignatureWithMetadata;
use serde::Deserialize;

use super::GenericResponseHandler;
use super::RequestHandler;

pub struct FourbyteClient {
    request_handler: RequestHandler,

    page_next_function: Option<String>,
    page_next_event: Option<String>,
}

#[derive(Deserialize)]
struct Page {
    next: Option<String>,
    results: Vec<FourbyteSignature>,

    #[serde(rename = "count")]
    _count: usize, // Used in the unit tests
}

#[derive(Deserialize)]
struct FourbyteSignature {
    text_signature: String,
}

impl FourbyteClient {
    /// Returns a new 4Byte API client.
    pub fn new() -> Self {
        FourbyteClient {
            request_handler: RequestHandler::new(),

            page_next_function: Some("https://www.4byte.directory/api/v1/signatures/?page=1".to_string()),
            page_next_event: Some("https://www.4byte.directory/api/v1/event-signatures/?page=1".to_string()),
        }
    }

    /// Returns the next function signature page, where the page index auto-increments internally with each
    /// function call.
    pub fn page_function_signature(&mut self) -> Result<Option<Vec<SignatureWithMetadata>>, Error> {
        if let Some(url) = self.page_next_function.as_ref() {
            let page = self.request_handler.execute_deser::<GenericResponseHandler, Page>(url)?;
            self.page_next_function = page.next;

            let mut signatures = Vec::new();
            for signature in page.results {
                signatures.push(SignatureWithMetadata::new(
                    signature.text_signature,
                    SignatureKind::Function,
                    true,
                ));
            }

            return Ok(Some(signatures));
        }

        Ok(None)
    }

    /// Returns the next event signature page, where the page index auto-increments internally with each
    /// function call.
    pub fn page_event_signature(&mut self) -> Result<Option<Vec<SignatureWithMetadata>>, Error> {
        if let Some(url) = self.page_next_event.as_ref() {
            let page = self.request_handler.execute_deser::<GenericResponseHandler, Page>(url)?;
            self.page_next_event = page.next;

            let mut signatures = Vec::new();
            for signature in page.results {
                signatures.push(SignatureWithMetadata::new(
                    signature.text_signature,
                    SignatureKind::Event,
                    true,
                ));
            }

            return Ok(Some(signatures));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::fourbyte::FourbyteClient;
    use crate::api::fourbyte::Page;
    use crate::api::GenericResponseHandler;

    fn page_signature_test(functions_endpoint: bool) {
        let url_page_01 = match functions_endpoint {
            true => "https://www.4byte.directory/api/v1/signatures/?page=1",
            false => "https://www.4byte.directory/api/v1/event-signatures/?page=1",
        };

        let url_page_02 = match functions_endpoint {
            true => "https://www.4byte.directory/api/v1/signatures/?page=2",
            false => "https://www.4byte.directory/api/v1/event-signatures/?page=2",
        };

        let http_client = reqwest::blocking::Client::default();
        let html_content_page01 = http_client.get(url_page_01).send().unwrap().text().unwrap();
        let html_content_page02 = http_client.get(url_page_02).send().unwrap().text().unwrap();

        let mut fbc = FourbyteClient::new();
        let fbc_signatures_page01 = match functions_endpoint {
            true => fbc.page_function_signature().unwrap().unwrap(),
            false => fbc.page_event_signature().unwrap().unwrap(),
        };

        let fbc_signatures_page02 = match functions_endpoint {
            true => fbc.page_function_signature().unwrap().unwrap(),
            false => fbc.page_event_signature().unwrap().unwrap(),
        };

        for signature in fbc_signatures_page01 {
            assert!(html_content_page01.contains(&signature.text));
        }

        for signature in fbc_signatures_page02 {
            assert!(html_content_page02.contains(&signature.text));
        }
    }

    #[test]
    fn page_function_signatures() {
        page_signature_test(true);
    }

    #[test]
    fn page_event_signatures() {
        page_signature_test(false);
    }

    fn page_count(signature_count: usize) -> usize {
        // We can calculate the total number of pages by checking if count / 100 has a remainder, if so
        // we have to round up otherwise it's OK. For example https://www.4byte.directory/api/v1/event-signatures/?page=1
        // has (currently) a count of 80032 with a total of 801 pages (80032 / 100 = 800 pages with 100
        // signatures + 1 page with 32 signatures). The 801th page contains 32 signatures which would be our
        // round up case.
        match signature_count % 100 {
            0 => signature_count / 100,
            _ => signature_count / 100 + 1,
        }
    }

    #[test]
    fn page_event_signatures_none() {
        let mut fbc = FourbyteClient::new();
        let page = fbc
            .request_handler
            .execute_deser::<GenericResponseHandler, Page>(fbc.page_next_event.as_ref().unwrap().as_ref())
            .unwrap();

        fbc.page_next_event = Some(format!(
            "https://www.4byte.directory/api/v1/event-signatures/?page={}",
            page_count(page._count)
        ));

        assert!(fbc.page_event_signature().unwrap().is_some()); // We're currently on the last page that contains signatures
        assert!(fbc.page_event_signature().unwrap().is_none()); // Calling this again we should get None
    }

    #[test]
    fn page_function_signatures_none() {
        let mut fbc = FourbyteClient::new();
        let page = fbc
            .request_handler
            .execute_deser::<GenericResponseHandler, Page>(fbc.page_next_function.as_ref().unwrap().as_ref())
            .unwrap();

        fbc.page_next_function =
            Some(format!("https://www.4byte.directory/api/v1/signatures/?page={}", page_count(page._count)));

        assert!(fbc.page_function_signature().unwrap().is_some()); // We're currently on the last page that contains signatures
        assert!(fbc.page_function_signature().unwrap().is_none()); // Calling this again we should get None
    }
}
