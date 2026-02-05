use reqwest::{Client, Error, Response, header::USER_AGENT};
use serde_json::Value;

use crate::models::{Listing, SavedPost};
use crate::rate_limiter::RateLimiter;

const DEFAULT_PAGE: u16 = 10;

/// A client through which posts are requested
pub struct RedditClient {
    pub client: Client,
    pub token: String,
    pub username: String,
    pub page_limit: u16,
    pub download_limit: u16,
    rate_limiter: RateLimiter,
}

impl RedditClient {
    pub fn new(token: String, username: String, download_limit: u16) -> Self {
        let client = reqwest::Client::new();

        let page_limit = if download_limit > DEFAULT_PAGE {
            DEFAULT_PAGE
        } else {
            download_limit
        };

        Self {
            client,
            token,
            username,
            page_limit,
            download_limit,
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Gets user information using an access token
    pub async fn get_saved_posts(&self) -> Result<Vec<SavedPost>, Error> {
        let mut post_count: usize = 0;
        let mut after = String::new();
        let mut posts = Vec::new();
        let url = format!("https://oauth.reddit.com/user/{}/saved", self.username);

        while post_count < self.download_limit as usize {
            let response = self.fetch_page(&url, &mut after, post_count).await?;
            let listing = response.json::<Listing<Value>>().await?;

            after = listing.data.after.unwrap();
            post_count += listing.data.children.len();

            for child in listing.data.children.into_iter() {
                if child.kind == "t3" {
                    match serde_json::from_value::<SavedPost>(child.data) {
                        Ok(post) => {
                            println!("Fetched {}--{} - {}", posts.len() + 1, post.id, post.title);
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

    async fn fetch_page(
        &self,
        url: &str,
        after: &str,
        post_count: usize,
    ) -> Result<Response, Error> {
        let download_limit = self.download_limit as usize;
        let page_limit = self.page_limit as usize;

        let limit = if (post_count + page_limit) <= download_limit {
            page_limit
        } else {
            download_limit - post_count
        };

        self.rate_limiter.wait().await;

        let request = self
            .client
            .get(url)
            .header(USER_AGENT, "rspd-script/0.1 by spencer")
            .query(&[("limit", &limit.to_string())])
            .query(&[("after", after)])
            .bearer_auth(&self.token);

        let response = request.send().await?;

        self.rate_limiter.update(response.headers());

        Ok(response)
    }
}
