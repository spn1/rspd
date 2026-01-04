use crate::models::SavedPost;
use anyhow::Error;
use sanitize_filename::sanitize;
use std::path::Path;

const SUPPORTED_EXTENSIONS: [&str; 5] = [".jpg", ".jpeg", ".png", ".gif", ".webp"];

pub async fn save_post(post: &SavedPost, base_output_path: &Path) -> Result<(), Error> {
    let subreddit_folder_name = sanitize(&post.subreddit);
    let target_dir = base_output_path.join(subreddit_folder_name);

    tokio::fs::create_dir_all(&target_dir).await?;

    if post.is_self {
        println!("{} is self post, skipping", post.id);
    } else {
        let is_direct_image = SUPPORTED_EXTENSIONS
            .iter()
            .any(|ext| post.url.ends_with(ext));

        if is_direct_image {
            let extension = Path::new(&post.url)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpg");

            let filename = format!("{}.{}", post.id, extension);
            let path = target_dir.join(filename);

            let image_response = reqwest::get(&post.url).await?;
            let bytes = image_response.bytes().await?;
            tokio::fs::write(path, bytes).await?;
            println!("Downloaded: {}", post.id);
        }
    }

    Ok(())
}
