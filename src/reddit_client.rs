use reqwest::{Client, Error, Response, header::USER_AGENT};
use serde_json::Value;

use crate::models::{Listing, SavedPost};

/// A client through which posts are requested
pub struct RedditClient {
    pub client: Client,
    pub token: String,
    pub username: String,
    pub limit: String,
}

impl RedditClient {
    pub fn new(client: Client, token: String, username: String) -> Self {
        Self {
            client,
            token,
            username,
            limit: String::from("1"),
        }
    }

    /// Gets user information using an access token
    pub async fn get_saved_posts(&self) -> Result<Vec<SavedPost>, Error> {
        const MAX_POSTS_TO_DOWNLOAD: usize = 10;
        let mut post_count: usize = 0;
        let mut after = String::new();
        let mut posts = Vec::new();
        let url = format!("https://oauth.reddit.com/user/{}/saved", self.username);

        while (post_count < MAX_POSTS_TO_DOWNLOAD) {
            let response = self.fetch_page(&url, &mut after).await?;
            let listing = response.json::<Listing<Value>>().await?;

            after = listing.data.after.unwrap();
            post_count = post_count + listing.data.children.len();

            for child in listing.data.children {
                if child.kind == "t3" {
                    match serde_json::from_value::<SavedPost>(child.data) {
                        Ok(post) => {
                            println!("Fetched {} - {}", post.id, post.title);
                            posts.push(post);
                        }
                        Err(error) => {
                            eprintln!("Failed to parse t3 saved item: {error}")
                        }
                    }
                }
            }
        }

        Ok(posts)
    }

    async fn fetch_page(&self, url: &str, after: &str) -> Result<Response, Error> {
        self.client
            .get(url)
            .header(USER_AGENT, "rspd-script/0.1 by neckbird")
            .query(&[("limit", &self.limit.to_string())])
            .query(&[("after", after)])
            .bearer_auth(&self.token)
            .send()
            .await
    }
}
