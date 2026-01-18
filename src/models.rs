use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
}

/// Reddit's "envelope" that contains data responses
#[derive(Serialize, Deserialize, Debug)]
pub struct Listing<T> {
    pub data: ListingData<T>,
}

/// The data returned, with pagination fields
#[derive(Serialize, Deserialize, Debug)]
pub struct ListingData<T> {
    after: Option<String>,
    before: Option<String>,
    pub children: Vec<Thing<T>>,
}

/// The saved thing, whether it is a post or comment.
#[derive(Serialize, Deserialize, Debug)]
pub struct Thing<T> {
    /// t3 = link/post, t1 = comment
    pub kind: String,
    pub data: T,
}

/// The content of the post.
#[derive(Deserialize, Debug)]
pub struct SavedPost {
    pub id: String,
    pub subreddit: String,
    pub url: String,
    pub is_gallery: Option<bool>,
    pub post_hint: Option<String>,
    pub media_metadata: Option<Value>,
    pub secure_media: Option<Value>, // videos
    pub is_self: bool,
}
