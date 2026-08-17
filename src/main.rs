use axum::{
    body::Body,
    http::Request,
    middleware::{self, Next},
    response::Response,
    Router,
    routing::get,
};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use percent_encoding::percent_decode_str;

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

fn decode_path(path: &str) -> String {
    if let Ok(decoded) = percent_decode_str(path).decode_utf8() {
        decoded.to_string()
    } else {
        path.to_string()
    }
}

fn extract_display_path(full_path: &str) -> String {
    if let Some(path_param) = full_path.strip_prefix("/raw?path=") {
        return decode_path(path_param);
    }
    if let Some(path_param) = full_path.strip_prefix("/viewer?path=") {
        return decode_path(path_param);
    }
    decode_path(full_path)
}

async fn log_request(
    root: &PathBuf, 
    full_path: &str, 
    status: u16, 
    latency_ms: u128,
    response_size: usize,
) {
    if full_path == "/favicon.ico" && status == 404 {
        return;
    }
    
    let time = Local::now().format("%H:%M:%S");
    let status_str = match status {
        200 | 201 | 204 => "✓",
        301 | 302 | 304 => "↷",
        400 | 401 => "⚠",
        403 | 404 | 500 => "✗",
        _ => "•",
    };
    
    let display_path = extract_display_path(full_path);
    
    let latency_str = if latency_ms > 0 {
        format!(" {}ms", latency_ms)
    } else {
        "".to_string()
    };
    
    let size_str = if response_size > 0 {
        if response_size > 1024 * 1024 {
            format!(" {:.1}MB", response_size as f64 / 1024.0 / 1024.0)
        } else if response_size > 1024 {
            format!(" {:.1}KB", response_size as f64 / 1024.0)
        } else {
            format!(" {}B", response_size)
        }
    } else {
        "".to_string()
    };
    
    let msg = format!("[{}] {} {} {}{}{}\n", 
        time, 
        status_str, 
        status, 
        display_path,
        latency_str,
        size_str
    );
    
    print!("{}", msg);
    
    // 日志文件输出到 root 目录（即 args.dir 指定的目录）
    let log_path = root.join("nwebp.log");
    if let Ok(mut file) = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path)
    {
        let _ = file.write_all(msg.as_bytes());
        let _ = file.flush();
    }
}

async fn log_middleware(
    state: axum::extract::State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("");
    let full_path = if query.is_empty() {
        path
    } else {
        format!("{}?{}", path, query)
    };
    
    let response = next.run(req).await;
    
    let latency = start.elapsed();
    let status = response.status().as_u16();
    
    let response_size = response.headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    
    // 从 state 获取 root_dir
    let root_dir = state.0.root_dir.clone();
    
    tokio::spawn(async move {
        log_request(
            &root_dir,
            &full_path,
            status,
            latency.as_millis(),
            response_size,
        ).await;
    });
    
    response
}

fn print_banner(root_dir: &PathBuf, host: &str, port: u16) {
    let version = env!("CARGO_PKG_VERSION");
    let banner = format!(r#"
╔═══════════════════════════════════════╗
║  📚 nwebp - 漫画浏览服务器 v{}     ║
║  🦀 Rust 强力驱动                     ║
╚═══════════════════════════════════════╝

📁 服务目录: {}
🌐 访问地址: http://{}:{}
📄 日志文件: {}/nwebp.log

💡 按 Ctrl+C 停止服务
"#, 
        version,
        root_dir.display(),
        host, 
        port,
        root_dir.display()
    );
    
    println!("{}", banner);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let root_dir = args.dir.canonicalize()?;
    
    let log_path = root_dir.join("nwebp.log");
    
    // 清空旧日志
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_path)
    {
        let _ = file.write_all(format!(
            "=== nwebp 启动 ===\n{}\nhttp://{}:{}\n{}\n\n",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            args.host,
            args.port,
            root_dir.display()
        ).as_bytes());
        let _ = file.flush();
    }
    
    // 打印启动横幅
    print_banner(&root_dir, &args.host, args.port);
    
    let state = Arc::new(AppState::new(root_dir));

    let app = Router::new()
        .route("/api/albums", get(handler::api_albums))
        .route("/api/album", get(handler::api_album_images))
        .route("/raw", get(handler::raw_image_handler))
        .route("/", get(handler::index_handler))
        .route("/viewer", get(handler::viewer_handler))
        .layer(middleware::from_fn_with_state(state.clone(), log_middleware))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;
    
    axum::serve(listener, app).await?;
    Ok(())
}