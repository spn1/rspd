use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
}

/// Reddit's "envelope" that contains data responses
#[derive(Deserialize, Debug)]
pub struct Listing<T> {
    pub data: ListingData<T>,
}

/// The data returned, with pagination fields
#[derive(Deserialize, Debug)]
pub struct ListingData<T> {
    after: Option<String>,
    before: Option<String>,
    pub children: Vec<Thing<T>>,
}

/// The data
#[derive(Deserialize, Debug)]
pub struct Thing<T> {
    /// t3 = link/post, t1 = comment
    pub kind: String,
    pub data: T,
}
