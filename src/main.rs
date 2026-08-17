use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod scanner;
mod handler;
mod template;

use scanner::AppState;

#[derive(Parser, Debug)]
#[command(name = "nwebp", version, about = "漫画图片在线浏览服务器")]
struct Args {
    #[arg(default_value = ".")]
    dir: PathBuf,
    #[arg(default_value_t = 8080)]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let root_dir = args.dir.canonicalize()?;
    println!("📂 漫画根目录: {}", root_dir.display());
    println!("🌐 监听地址: http://{}:{}", args.host, args.port);

    let state = Arc::new(AppState::new(root_dir));

    let app = axum::Router::new()
        .route("/api/albums", axum::routing::get(handler::api_albums))
        .route("/api/album", axum::routing::get(handler::api_album_images))
        .route("/raw", axum::routing::get(handler::raw_image_handler))
        .route("/", axum::routing::get(handler::index_handler))
        .route("/viewer", axum::routing::get(handler::viewer_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("✅ 服务已启动，按 Ctrl+C 停止");

    axum::serve(listener, app).await?;
    Ok(())
}