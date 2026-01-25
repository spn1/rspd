mod downloader;
mod models;
mod reddit_client;

use anyhow::Error;
use clap::Parser;
use dotenvy::dotenv;
use downloader::save_posts;
use models::TokenResponse;
use reddit_client::RedditClient;
use reqwest::header::USER_AGENT;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of previous saved posts to save.
    #[arg(short, long, default_value_t = 10)]
    download_imit: u16,
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
    // Parse command line arguments
    let args = Args::parse();
    let download_limit = args.download_imit;
    let request_limit = if download_limit > 5 {
        5
    } else {
        download_limit
    };

    // Get Env Vars
    dotenv().ok();
    let client_id = std::env::var("REDDIT_CLIENT_ID").expect("Missing REDDIT_CLIENT_ID");
    let client_secret =
        std::env::var("REDDIT_CLIENT_SECRET").expect("Missing REDDIT_CLIENT_SECRET");
    let username = std::env::var("REDDIT_USERNAME").expect("Missing REDDIT_USERNAME");
    let password = std::env::var("REDDIT_PASSWORD").expect("Missing REDDIT_PASSWORD");

    // Get reddit access token
    let access_token = get_access_token(&client_id, &client_secret, &username, &password).await?;

    // Fetch all posts
    let client = reqwest::Client::new();
    let reddit_client = RedditClient::new(
        client,
        access_token,
        username,
        request_limit,
        download_limit,
    );
    let saved_posts = reddit_client.get_saved_posts().await?;

    // download fetched posts
    save_posts(&saved_posts).await?;

    Ok(())
}
