mod downloader;
mod models;
mod rate_limiter;
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
    download_limit: u16,

    /// The Client ID for this reddit application
    #[arg(long, env = "REDDIT_CLIENT_ID")]
    reddit_client_id: Option<String>,

    /// The Client Secret for this reddit application
    #[arg(long, env = "REDDIT_CLIENT_SECRET")]
    reddit_client_secret: Option<String>,

    /// Your reddit username
    #[arg(long, env = "REDDIT_USERNAME")]
    reddit_username: Option<String>,

    /// Your reddit password
    #[arg(long, env = "REDDIT_PASSWORD")]
    reddit_password: Option<String>,
}

struct Options {
    download_limit: u16,
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
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

/// Get options for running the program via command line arguments to environment variables
fn get_options() -> Result<Options, Error> {
    dotenv().ok();
    let args = Args::parse();

    let client_id = args.reddit_client_id.expect("Missing REDDIT_CLIENT_ID");
    let client_secret = args
        .reddit_client_secret
        .expect("Missing REDDIT_CLIENT_SECRET");
    let username = args.reddit_username.expect("Missing REDDIT_USERNAME");
    let password = args.reddit_password.expect("Missing REDDIT_PASSWORD");
    let download_limit = std::env::var("REDDIT_DOWNLOAD_LIMIT")
        .expect("Missing REDDIT_DOWNLOAD_LIMIT")
        .parse::<u16>()
        .unwrap();

    Ok(Options {
        client_id,
        client_secret,
        username,
        password,
        download_limit,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let Options {
        client_id,
        client_secret,
        username,
        password,
        download_limit,
    } = get_options().unwrap();

    // Get reddit access token
    let access_token = get_access_token(&client_id, &client_secret, &username, &password).await?;

    // Fetch all posts
    let reddit_client = RedditClient::new(access_token, username, download_limit);
    let saved_posts = reddit_client.get_saved_posts().await?;

    // download fetched posts
    save_posts(&saved_posts).await?;

    Ok(())
}
