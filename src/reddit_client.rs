use reqwest::{Client, Error, header::USER_AGENT};
use serde_json::Value;

use crate::models::Listing;

pub struct RedditClient {
    pub client: Client,
    pub token: String,
    pub username: String,
}

impl RedditClient {
    pub fn new(client: Client, token: String, username: String) -> Self {
        Self {
            client,
            token,
            username,
        }
    }

    /// Gets user information using an access token
    pub async fn get_saved_posts(&self) -> Result<Listing<Value>, Error> {
        let url = format!(
            "https://oauth.reddit.com/user/{}/saved?limit=5",
            self.username
        );

        let response = self
            .client
            .get(url)
            .header(USER_AGENT, "rspd-script/0.1 by neckbird")
            .bearer_auth(&self.token)
            .send()
            .await?;

        let listing = response.json::<Listing<Value>>().await?;

        Ok(listing)
    }
}
