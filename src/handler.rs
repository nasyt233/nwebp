use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Json, Response};
use serde::Deserialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

use crate::scanner::AppState;
use crate::template;

type AppStateRef = State<Arc<AppState>>;

#[derive(Deserialize)]
pub struct PathQuery {
    #[serde(default)]
    pub path: String,
}

pub async fn index_handler(
    State(state): AppStateRef,
    Query(q): Query<PathQuery>,
) -> Html<String> {
    let browse = state.scan_folder(&q.path).await;
    let config = state.get_config().await;
    let views = state.get_all_views().await;
    Html(template::render_index(&browse, &config, &views))
}

#[derive(Deserialize)]
pub struct ViewerQuery { pub path: String }

pub async fn viewer_handler(
    State(state): AppStateRef,
    Query(query): Query<ViewerQuery>,
) -> Response {
    match state.get_album_images(&query.path).await {
        Some(album) => {
            // 后端统计浏览次数
            state.increment_view(&query.path).await;
            Html(template::render_viewer(&album)).into_response()
        }
        None => (StatusCode::NOT_FOUND, "本子不存在").into_response(),
    }
}

/// GET /api/albums - 所有漫画（含 views）
pub async fn api_albums(State(state): AppStateRef) -> Json<serde_json::Value> {
    let albums = state.scan_albums().await;
    let views = state.get_all_views().await;
    let total_size: u64 = albums.iter().map(|a| a.size).sum();

    let albums_json: Vec<serde_json::Value> = albums.into_iter().map(|a| {
        let v = views.get(&a.path).copied().unwrap_or(0);
        serde_json::json!({
            "name": a.name,
            "path": a.path,
            "image_count": a.image_count,
            "cover": a.cover,
            "modified": a.modified,
            "size": a.size,
            "views": v,
        })
    }).collect();

    Json(serde_json::json!({
        "count": albums_json.len(),
        "total_size": total_size,
        "albums": albums_json,
    }))
}

/// GET /api/album?path=xxx - 单个漫画图片列表
#[derive(Deserialize)]
pub struct AlbumQuery { pub path: String }

pub async fn api_album_images(
    State(state): AppStateRef,
    Query(query): Query<AlbumQuery>,
) -> Response {
    match state.get_album_images(&query.path).await {
        Some(album) => Json(album).into_response(),
        None => (StatusCode::NOT_FOUND, "本子不存在").into_response(),
    }
}

/// GET /api/folder?path=xxx - 文件夹浏览（含 views）
pub async fn api_folder(
    State(state): AppStateRef,
    Query(q): Query<PathQuery>,
) -> Json<serde_json::Value> {
    let browse = state.scan_folder(&q.path).await;
    let views = state.get_all_views().await;
    let albums_json: Vec<serde_json::Value> = browse.albums.into_iter().map(|a| {
        let v = views.get(&a.path).copied().unwrap_or(0);
        serde_json::json!({
            "name": a.name, "path": a.path, "image_count": a.image_count,
            "cover": a.cover, "modified": a.modified, "size": a.size, "views": v,
        })
    }).collect();
    Json(serde_json::json!({
        "current_path": browse.current_path,
        "folders": browse.folders,
        "albums": albums_json,
        "breadcrumbs": browse.breadcrumbs,
    }))
}

/// GET /api/stats - 库统计
pub async fn api_stats(State(state): AppStateRef) -> Json<serde_json::Value> {
    let albums = state.scan_albums().await;
    let views = state.get_all_views().await;
    let total_albums = albums.len();
    let total_images: usize = albums.iter().map(|a| a.image_count).sum();
    let total_size: u64 = albums.iter().map(|a| a.size).sum();
    let total_views: u64 = views.values().sum();
    Json(serde_json::json!({
        "total_albums": total_albums,
        "total_images": total_images,
        "total_size": total_size,
        "total_views": total_views,
    }))
}

/// GET /api/views - 所有浏览次数
pub async fn api_views(State(state): AppStateRef) -> Json<serde_json::Value> {
    let views = state.get_all_views().await;
    Json(serde_json::json!({ "views": views }))
}

/// GET /api/random - 随机推荐
pub async fn api_random(State(state): AppStateRef) -> Json<serde_json::Value> {
    let albums = state.scan_albums().await;
    if albums.is_empty() {
        return Json(serde_json::json!({ "album": null }));
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0);
    let idx = seed % albums.len();
    Json(serde_json::json!({ "album": albums[idx] }))
}

#[derive(Deserialize)]
pub struct RawQuery { pub path: String }

pub async fn raw_image_handler(
    State(state): AppStateRef,
    Query(query): Query<RawQuery>,
) -> Response {
    let rel_path = query.path.trim_start_matches('/');
    debug!("raw request: {}", rel_path);
    match state.resolve_path(rel_path) {
        Some(full_path) => match File::open(&full_path).await {
            Ok(mut file) => {
                let mut buf = Vec::new();
                if let Err(e) = file.read_to_end(&mut buf).await {
                    error!("read_to_end failed: {}", e);
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
                error!("file open failed: {}", e);
                (StatusCode::NOT_FOUND, "文件不存在").into_response()
            }
        },
        None => {
            error!("resolve_path returned None for: {}", rel_path);
            (StatusCode::NOT_FOUND, "文件不存在或路径非法").into_response()
        }
    }
}

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