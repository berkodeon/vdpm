use crate::error::Result;
use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

pub struct GithubClient {
    client: reqwest::Client,
}

impl GithubClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("vdpm-client"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to build HTTP client");

        Self { client }
    }

    pub async fn download(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("failed to reach \"{url}\""))?;

        let response = response
            .error_for_status()
            .with_context(|| format!("received an error response from \"{url}\""))?;

        response
            .text()
            .await
            .with_context(|| format!("failed to read response body from \"{url}\""))
    }
}
