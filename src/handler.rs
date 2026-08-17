use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Json, Response};
use serde::Deserialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::scanner::AppState;
use crate::template;

type AppStateRef = State<Arc<AppState>>;

/// 首页 - 列表
pub async fn index_handler(State(state): AppStateRef) -> Html<String> {
    let albums = state.scan_albums().await;
    Html(template::render_index(&albums))
}

/// 阅读页查询参数
#[derive(Deserialize)]
pub struct ViewerQuery {
    pub path: String,
}

pub async fn viewer_handler(
    State(state): AppStateRef,
    Query(query): Query<ViewerQuery>,
) -> Response {
    match state.get_album_images(&query.path).await {
        Some(album) => Html(template::render_viewer(&album)).into_response(),
        None => (StatusCode::NOT_FOUND, "本子不存在").into_response(),
    }
}

/// API: 获取所有列表
pub async fn api_albums(State(state): AppStateRef) -> Json<serde_json::Value> {
    let albums = state.scan_albums().await;
    Json(serde_json::json!({
        "count": albums.len(),
        "albums": albums,
    }))
}

/// API: 获取图片列表
#[derive(Deserialize)]
pub struct AlbumQuery {
    pub path: String,
}

pub async fn api_album_images(
    State(state): AppStateRef,
    Query(query): Query<AlbumQuery>,
) -> Response {
    match state.get_album_images(&query.path).await {
        Some(album) => Json(album).into_response(),
        None => (StatusCode::NOT_FOUND, "本子不存在").into_response(),
    }
}

/// 原始图片服务（path 通过 query 参数传递）
#[derive(Deserialize)]
pub struct RawQuery {
    pub path: String,
}

pub async fn raw_image_handler(
    State(state): AppStateRef,
    Query(query): Query<RawQuery>,
) -> Response {
    let rel_path = query.path.trim_start_matches('/');
    eprintln!("[DEBUG] raw_image_handler request: {}", rel_path);

    match state.resolve_path(rel_path) {
        Some(full_path) => {
            eprintln!("[DEBUG] resolved to: {:?}", full_path);
            match File::open(&full_path).await {
                Ok(mut file) => {
                    let mut buf = Vec::new();
                    if let Err(e) = file.read_to_end(&mut buf).await {
                        eprintln!("[ERROR] read_to_end failed: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "读取文件失败").into_response();
                    }
                    let content_type = guess_content_type(&full_path);
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, content_type)
                        .header(header::CACHE_CONTROL, "public, max-age=604800")
                        .header(header::ACCEPT_RANGES, "bytes")
                        .body(buf.into())
                        .unwrap()
                }
                Err(e) => {
                    eprintln!("[ERROR] file open failed: {}", e);
                    (StatusCode::NOT_FOUND, "文件不存在").into_response()
                }
            }
        }
        None => {
            eprintln!("[ERROR] resolve_path returned None for: {}", rel_path);
            (StatusCode::NOT_FOUND, "文件不存在或路径非法").into_response()
        }
    }
}

/// 根据扩展名猜测 Content-Type
fn guess_content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) {
        Some(ref ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ref ext) if ext == "png" => "image/png",
        Some(ref ext) if ext == "webp" => "image/webp",
        Some(ref ext) if ext == "gif" => "image/gif",
        Some(ref ext) if ext == "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}