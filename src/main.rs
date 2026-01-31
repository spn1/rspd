mod downloader;
mod models;
mod reddit_client;

use anyhow::Error;
use clap::Parser;
use downloader::save_posts;
use models::TokenResponse;
use reddit_client::RedditClient;
use reqwest::header::USER_AGENT;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of previous saved posts to save.
    #[arg(short, long, default_value_t = 10)]
    download_limit: u16,

    /// The Client ID for this reddit application
    #[arg(long)]
    reddit_client_id: String,

    /// The Client Secret for this reddit application
    #[arg(long)]
    reddit_client_secret: String,

    /// Your reddit username
    #[arg(long)]
    reddit_username: String,

    /// Your reddit password
    #[arg(long)]
    reddit_password: String,
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    // Get reddit access token
    let access_token = get_access_token(
        &args.reddit_client_id,
        &args.reddit_client_secret,
        &args.reddit_username,
        &args.reddit_password,
    )
    .await?;

    // Fetch all posts
    let reddit_client = RedditClient::new(access_token, args.reddit_username, args.download_limit);
    let saved_posts = reddit_client.get_saved_posts().await?;

    // download fetched posts
    save_posts(&saved_posts).await?;

    Ok(())
}
