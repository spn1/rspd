use crate::models::SavedPost;
use anyhow::Error;
use sanitize_filename::sanitize;
use std::path::{Path, PathBuf};

/// Supported file extensions for images.
const SUPPORTED_IMAGE_EXTENSIONS: [&str; 5] = [".jpg", ".jpeg", ".png", ".gif", ".webp"];

/// Saves a list of posts to local directories
pub async fn save_posts(posts: &Vec<SavedPost>) -> Result<(), Error> {
    for post in posts.iter() {
        if post.is_self {
            println!("{} is self post, skipping", post.id);
        } else {
            save_post(post, Path::new("saved")).await?;
        }
    }

    Ok(())
}

/// Saves a single post to a local directory
pub async fn save_post(post: &SavedPost, base_output_path: &Path) -> Result<(), Error> {
    let subreddit_folder_name = sanitize(&post.subreddit);
    let target_dir = base_output_path.join(subreddit_folder_name);

    tokio::fs::create_dir_all(&target_dir).await?;

    if post.is_gallery == Some(true) {
        handle_gallery(post, target_dir).await?;
    } else if let Some("hosted:video") = post.post_hint.as_deref() {
        handle_video(post, target_dir).await?;
    } else {
        handle_image(post, target_dir).await?;
    }

    Ok(())
}

/// Handles download logic when post is a single video
pub async fn handle_video(post: &SavedPost, target_dir: PathBuf) -> Result<(), Error> {
    if let Some(video_info) = post
        .secure_media
        .as_ref()
        .and_then((|sm| sm.get("reddit_video")))
    {
        if let Some(fallback_url) = video_info.get("fallback_url").and_then(|u| u.as_str()) {
            let media_filename = format!("{}.mp4", post.id);
            download_file(fallback_url, &target_dir.join(media_filename)).await?;
        }
    }

    Ok(())
}

/// Handles download logic when post is a gallery of images.
pub async fn handle_gallery(post: &SavedPost, target_dir: PathBuf) -> Result<(), Error> {
    if let Some(media_meta) = &post.media_metadata {
        if let Some(items) = media_meta.as_object() {
            let mut count = 1;
            for (_media_id, item_data) in items {
                if let Some(item_url) = item_data.get("s").and_then(|s| s.get("u")) {
                    if let Some(item_url_str) = item_url.as_str() {
                        let clean_url = item_url_str.replace("&amp;", "&");
                        let extension = Path::new(&clean_url)
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("jpg");

                        let media_filename = format!("{}_gallery_{}.{}", post.id, count, extension);
                        download_file(&clean_url, &target_dir.join(media_filename)).await?;
                        println!("Downloaded gallery image {} for post {}", count, post.id);
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handles download logic when post is a single image.
pub async fn handle_image(post: &SavedPost, target_dir: PathBuf) -> Result<(), Error> {
    let is_direct_image = SUPPORTED_IMAGE_EXTENSIONS
        .iter()
        .any(|ext| post.url.ends_with(ext));

    if is_direct_image {
        let filename = get_filename(post);
        let path = target_dir.join(filename);
        download_file(&post.url, &path).await?;
        println!("Downloaded: {}", post.id);
    }

    Ok(())
}

/// Downloads a file from the given URL and saves it to the given path.
pub async fn download_file(url: &str, path: &Path) -> Result<(), Error> {
    let image_response = reqwest::get(url).await?;

    if !image_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download from {}: status {}",
            url,
            image_response.status()
        ));
    }

    let bytes = image_response.bytes().await?;
    tokio::fs::write(path, bytes).await?;

    Ok(())
}

/// Returns a name for a saved post, composed of the post ID and the appropriate extension.
pub fn get_filename(post: &SavedPost) -> String {
    let extension = Path::new(&post.url)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg");

    format!("{}.{}", post.id, extension)
}
