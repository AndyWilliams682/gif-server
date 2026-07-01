use axum::{extract::Path, http::StatusCode, response::{Html, IntoResponse}, routing::get, Router};
use tower_http::services::ServeDir;
use maud::html;
use std::path::PathBuf;

const MOUNTED_DIR: &str = "/mnt/gifs";

#[tokio::main]
async fn main() {
    // 1. Mount raw content files to an internal routing endpoint
    let static_media = ServeDir::new(MOUNTED_DIR);

    // 2. Formulate paths
    let app = Router::new()
        .nest_service("/raw", static_media)      // Fetches raw bytes (e.g., /raw/cat.gif)
        .route("/", get(build_gallery))          // Serves root gallery index
        .route("/:gif_name", get(serve_page));   // Serves dynamic wrapper shell

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_page(Path(gif_name): Path<String>) -> impl IntoResponse {
    // Sanitize path parameter to prevent path traversal exploits (../)
    let sanitized_path = PathBuf::from(MOUNTED_DIR).join(format!("{}.gif", gif_name));
    
    if !sanitized_path.exists() {
        return (StatusCode::NOT_FOUND, "Resource Missing").into_response();
    }

    // Compile type-safe HTML strings natively at runtime with Open Graph metadata
    let html_page = html! {
        (maud::DOCTYPE)
        html {
            head {
                title { (gif_name) }
                
                // 1. Tell Discord this page hosts a rich video/image asset
                meta property="og:type" content="video.other"; 
                
                // 2. Give Discord the DIRECT, absolute URL to the raw GIF file
                // Replace "gifs.ampersan.de" with your exact subdomain/domain setup
                meta property="og:image" content=(format!("https://gifs.ampersan.de/raw/{}.gif", gif_name));
                
                // 3. Optimize the layout size for large media blocks on Discord/Twitter
                meta name="twitter:card" content="summary_large_image";
                meta property="og:image:type" content="image/gif";

                style { "body { margin:0; background:#0b0b0b; display:flex; justify-content:center; align-items:center; height:100vh; } img { max-width:100%; max-height:100vh; object-fit:contain; }" }
            }
            body {
                img src=(format!("/raw/{}.gif", gif_name)) alt=(gif_name);
            }
        }
    };

async fn build_gallery() -> impl IntoResponse {
    // Read the mount directly on request, making addition of new files instantaneous
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(MOUNTED_DIR) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() && entry.path().extension().map_or(false, |ext| ext == "gif") {
                    files.push(entry.file_name().to_string_lossy().into_owned());
                }
            }
        }
    }
    // ... Render list layout using maud macro loops
    Html("Gallery content generated here").into_response()
}
