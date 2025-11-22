use reqwest::{Error, header::USER_AGENT};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    name: String,
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
async fn get_user_info(access_token: &str) -> Result<ApiResponse, Error> {
    let url = "https://oauth.reddit.com/api/v1/me";
    let client = reqwest::Client::new();

    let request = client
        .get(url)
        .header(USER_AGENT, "rspd-script/0.1 by neckbird")
        .bearer_auth(access_token);

    let response = request.send().await?;
    let json = response.json::<ApiResponse>().await?;

    Ok(json)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client_id = std::env::var("REDDIT_CLIENT_ID").expect("Missing REDDIT_CLIENT_ID");
    let client_secret =
        std::env::var("REDDIT_CLIENT_SECRET").expect("Missing REDDIT_CLIENT_SECRET");
    let username = std::env::var("REDDIT_USERNAME").expect("Missing REDDIT_USERNAME");
    let password = std::env::var("REDDIT_PASSWORD").expect("Missing REDDIT_PASSWORD");

    let access_token = get_access_token(&client_id, &client_secret, &username, &password).await?;
    let user_info = get_user_info(&access_token).await?;

    println!("{}", user_info.name);

    Ok(())
}
