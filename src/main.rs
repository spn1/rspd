use reqwest::{Error, header::USER_AGENT};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
}

/// Reddit's "envelope" that contains data responses
#[derive(Deserialize, Debug)]
struct Listing<T> {
    data: ListingData<T>,
}

/// The data returned, with pagination fields
#[derive(Deserialize, Debug)]
struct ListingData<T> {
    after: Option<String>,
    before: Option<String>,
    children: Vec<Thing<T>>,
}

/// The data
#[derive(Deserialize, Debug)]
struct Thing<T> {
    /// t3 = link/post, t1 = comment
    kind: String,
    data: T,
}

/// Gets an access token for the application and user account from reddit API
async fn get_access_token(
    client_id: &str,
    client_secret: &str,
    username: &str,
    password: &str,
) -> Result<String, Error> {
    let url = "https://www.reddit.com/api/v1/access_token";
    let mock_form = [
        ("grant_type", "password"),
        ("username", username),
        ("password", password),
    ];
    let client = reqwest::Client::new();

    // Make the POST request and wait for the response
    let response = client
        .post(url)
        .form(&mock_form)
        .header(USER_AGENT, "rspd-script/0.1")
        .basic_auth(client_id, Some(client_secret))
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    Ok(response.access_token)
}

/// Gets user information using an access token
async fn get_saved_posts(access_token: &str, username: &str) -> Result<Listing<Value>, Error> {
    let url = format!("https://oauth.reddit.com/user/{username}/saved?limit=10");
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header(USER_AGENT, "rspd-script/0.1 by neckbird")
        .bearer_auth(access_token)
        .send()
        .await?;

    let listing = response.json::<Listing<Value>>().await?;

    Ok(listing)
}

fn debug_posts(posts: &Listing<Value>) {
    for child in &posts.data.children {
        match child.kind.as_str() {
            // Post / Link
            "t3" => {
                let id = child.data.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let title = child
                    .data
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let subreddit = child
                    .data
                    .get("subreddit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                println!("POST t3 id={} subreddit={} title={}", id, subreddit, title);
            }
            // Comment
            "t1" => {
                let id = child.data.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let body = child
                    .data
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let subreddit = child
                    .data
                    .get("subreddit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                println!("POST t3 id={} subreddit={} body={}", id, subreddit, body);
            }
            other => {
                println!("Other kind={}", other)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client_id = std::env::var("REDDIT_CLIENT_ID").expect("Missing REDDIT_CLIENT_ID");
    let client_secret =
        std::env::var("REDDIT_CLIENT_SECRET").expect("Missing REDDIT_CLIENT_SECRET");
    let username = std::env::var("REDDIT_USERNAME").expect("Missing REDDIT_USERNAME");
    let password = std::env::var("REDDIT_PASSWORD").expect("Missing REDDIT_PASSWORD");

    let access_token = get_access_token(&client_id, &client_secret, &username, &password).await?;
    let saved_posts = get_saved_posts(&access_token, &username).await?;

    debug_posts(&saved_posts);

    Ok(())
}
