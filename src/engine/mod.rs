mod bing;
mod duckduckgo;
mod google;

use crate::config::SearchEngine;
use crate::error::{Error, Result};
use crate::utils::random_agent;
use reqwest::{Client, ClientBuilder, RequestBuilder};

/// Search result links under the given search engine.
///
/// This function will go through network to find out useful links.
///
/// # Examples
///
/// ```rust
/// # async fn run() {
/// use std::str::FromStr;
/// use hors::{self, SearchEngine};
///
/// let search_engine: SearchEngine = SearchEngine::from_str("bing").unwrap();
/// let target_links: Vec<String> = hors::search_links(
///     "how to parse json in rust",
///     search_engine
/// )
/// .await
/// .unwrap();
/// assert_ne!(target_links.len(), 0);
/// for link in target_links {
///     assert!(link.contains("stackoverflow.com"))
/// }
/// # }
/// ```
pub async fn search_links(query: &str, search_engine: SearchEngine) -> Result<Vec<String>> {
    let client: Client = ClientBuilder::new().cookie_store(true).build()?;

    search_links_with_client(query, search_engine, &client).await
}

/// Search result links under the given search engine.
///
/// This function will go through network to find out useful links.
///
/// # Examples
///
/// ```rust
/// use std::str::FromStr;
/// use hors::{self, Config, OutputOption, Result, SearchEngine};
/// use reqwest::{Client, ClientBuilder};
///
/// # async fn run() {
/// let search_engine: SearchEngine = SearchEngine::from_str("bing").unwrap();
/// // please make sure that `cookie_store` should set to `true` in client builder.
/// let mut client: Client = ClientBuilder::new().cookie_store(true).build().unwrap();
/// let target_links: Vec<String> = hors::search_links_with_client(
///     "how to parse json in rust",
///     search_engine,
///     &client
/// )
/// .await
/// .unwrap();
/// assert_ne!(target_links.len(), 0);
/// for link in target_links {
///     assert!(link.contains("stackoverflow.com"));
/// }
/// # }
/// ```
///
/// # Returns
///
/// If search links successfully, it will return a Vector of String, which indicate
/// relative links to got answer.  Else return an Error.
pub async fn search_links_with_client(
    query: &str,
    search_engine: SearchEngine,
    client: &Client,
) -> Result<Vec<String>> {
    let https_opts: Vec<bool> = vec![true, false];
    for opt in https_opts {
        let fetch_url: String = get_query_url(query, &search_engine, opt);
        let page: String = fetch(&fetch_url, client).await?;
        let extract_results = extract_links(&page, &search_engine);
        if let Some(links) = extract_results {
            return Ok(links);
        }
    }
    Err(Error::from_parse("Can't find search result..."))
}

fn get_query_url(query: &str, search_engine: &SearchEngine, use_https: bool) -> String {
    match search_engine {
        SearchEngine::Bing => bing::get_query_url(query, use_https),
        SearchEngine::Google => google::get_query_url(query, use_https),
        SearchEngine::DuckDuckGo => duckduckgo::get_query_url(query, use_https),
    }
}

/// Fetch actual page according to given url.
///
/// # Arguments
///
/// * `search_url` - The url which should lead to search result page.
/// * `client` - An instance of `request::Client` object which can use to fire http request,
///              please ensure that it's build with cookie_store(true) option.
///
/// # Returns
///
/// If get search result page successfully, it will return the content of page,
/// or returns error.
async fn fetch(search_url: &str, client: &Client) -> Result<String> {
    let request: RequestBuilder = client
        .get(search_url)
        .header(reqwest::header::USER_AGENT, random_agent());
    debug!("Request to bing information: {:?}", request);
    let res = request.send().await?;
    let page: String = res.text().await?;
    Ok(page)
}

/// Extract links from given page.
///
/// # Arguments
///
/// * `page` - the search result page, which is mainly got by `fetch` function.
/// * `search_engine` - indicate which search engine we can use to extract links out.
///
/// # Returns
///
/// Links to the relative question, or returns None if we can't find it.
fn extract_links(page: &str, search_engine: &SearchEngine) -> Option<Vec<String>> {
    match search_engine {
        SearchEngine::Bing => bing::extract_links(page),
        SearchEngine::Google => google::extract_links(page),
        SearchEngine::DuckDuckGo => duckduckgo::extract_links(page),
    }
}
