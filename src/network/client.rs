use reqwest::Client;
use std::env;

pub struct HttpClient {
    pub client: Client,
    pub token: String,
}

impl HttpClient {
    pub fn new() -> color_eyre::Result<Self> {
        let token = std::env::var("OPENF1_TOKEN").unwrap_or_default();
        let client = Client::builder()
            .rustls_client()
            .build()?;
        Ok(Self { client, token })
    }

    pub fn with_token(&self, url: &str) -> String {
        if !self.token.is_empty() {
            format!("{}&token={}", url, self.token)
        } else {
            url.to_string()
        }
    }
}