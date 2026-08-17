use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use walkdir::WalkDir;

/// 图片扩展名
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];
const CACHE_TTL: Duration = Duration::from_secs(5);

// ---------- 配置结构 ----------
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub title: String,
    pub subtitle: String,
    pub show_cover: bool,
    pub primary_color: String,
    pub footer: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "📚 nwebp 漫画库".to_string(),
            subtitle: "轻量级网页漫画阅读器".to_string(),
            show_cover: true,
            primary_color: "#667eea".to_string(),
            footer: "⚡ 由 Rust 强力驱动 · nwebp 漫画浏览".to_string(),
        }
    }
}

impl Config {
    /// 从文件加载配置，若不存在则创建默认
    pub fn load_from_file(root: &Path) -> Self {
        let path = root.join("nwebp.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&content) {
                return cfg;
            }
        }
        let cfg = Config::default();
        let _ = std::fs::write(&path, serde_json::to_string_pretty(&cfg).unwrap());
        cfg
    }
}

// ---------- 应用状态 ----------
pub struct AppState {
    pub root_dir: PathBuf,
    pub config: Arc<RwLock<Config>>,
    albums_cache: Arc<Mutex<Option<(Vec<Album>, Instant)>>>,
    images_cache: Arc<Mutex<HashMap<String, (AlbumImages, Instant)>>>,
}

impl AppState {
    pub fn new(root_dir: PathBuf) -> Self {
        let config = Arc::new(RwLock::new(Config::load_from_file(&root_dir)));
        Self {
            root_dir,
            config,
            albums_cache: Arc::new(Mutex::new(None)),
            images_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取配置（只读）
    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn scan_albums(&self) -> Vec<Album> {
        {
            let cache = self.albums_cache.lock().await;
            if let Some((albums, time)) = cache.as_ref() {
                if time.elapsed() < CACHE_TTL {
                    return albums.clone();
                }
            }
        }

        let root_dir = self.root_dir.clone();
        let albums = tokio::task::spawn_blocking(move || do_scan_albums(&root_dir))
            .await
            .unwrap();

        let mut cache = self.albums_cache.lock().await;
        *cache = Some((albums.clone(), Instant::now()));
        albums
    }

    pub async fn get_album_images(&self, album_path: &str) -> Option<AlbumImages> {
        {
            let cache = self.images_cache.lock().await;
            if let Some((images, time)) = cache.get(album_path) {
                if time.elapsed() < CACHE_TTL {
                    return Some(images.clone());
                }
            }
        }

        let root_dir = self.root_dir.clone();
        let path = album_path.to_string();
        let result = tokio::task::spawn_blocking(move || do_get_album_images(&root_dir, &path))
            .await
            .unwrap();

        if let Some(images) = result {
            let mut cache = self.images_cache.lock().await;
            cache.insert(album_path.to_string(), (images.clone(), Instant::now()));
            Some(images)
        } else {
            None
        }
    }

    /// 安全解析文件路径
    pub fn resolve_path(&self, rel_path: &str) -> Option<PathBuf> {
        if rel_path.is_empty() {
            return None;
        }
        let full = self.root_dir.join(rel_path);
        if let Ok(canon) = full.canonicalize() {
            if canon.starts_with(&self.root_dir) && canon.is_file() {
                return Some(canon);
            }
        }
        let normalized = full.components().collect::<PathBuf>();
        if normalized.starts_with(&self.root_dir) && normalized.is_file() {
            Some(normalized)
        } else {
            None
        }
    }
}

// ---------- 同步扫描函数 ----------
fn do_scan_albums(root_dir: &Path) -> Vec<Album> {
    let mut albums = Vec::new();
    for entry in WalkDir::new(root_dir).min_depth(1).max_depth(3) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let mut images = list_images(path);
        if images.is_empty() {
            continue;
        }
        // 自然排序，确保第一张是真正的首图
        images.sort_by(|a, b| {
            let a_name = a.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            let b_name = b.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            natord::compare(&a_name, &b_name)
        });

        let relative = path.strip_prefix(root_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| relative.clone());
        let cover = images.first().map(|p| {
            p.strip_prefix(root_dir)
                .unwrap_or(p)
                .to_string_lossy()
                .to_string()
        });
        albums.push(Album {
            name,
            path: relative,
            image_count: images.len(),
            cover,
        });
    }
    albums.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    albums
}

fn do_get_album_images(root_dir: &Path, album_path: &str) -> Option<AlbumImages> {
    let full_path = root_dir.join(album_path);
    if !full_path.is_dir() {
        return None;
    }
    let mut images = list_images(&full_path);
    images.sort_by(|a, b| {
        let a_name = a.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        let b_name = b.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        natord::compare(&a_name, &b_name)
    });
    let relative_images: Vec<String> = images
        .iter()
        .map(|p| {
            p.strip_prefix(root_dir)
                .unwrap_or(p)
                .to_string_lossy()
                .to_string()
        })
        .collect();
    let name = full_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| album_path.to_string());
    Some(AlbumImages {
        name,
        path: album_path.to_string(),
        images: relative_images,
    })
}

// ---------- 数据结构 ----------
#[derive(Serialize, Clone)]
pub struct Album {
    pub name: String,
    pub path: String,
    pub image_count: usize,
    pub cover: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct AlbumImages {
    pub name: String,
    pub path: String,
    pub images: Vec<String>,
}

// ---------- 辅助函数 ----------
fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn list_images(dir: &Path) -> Vec<PathBuf> {
    let mut images = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() && is_image(&path) {
                images.push(path);
            }
        }
    }
    images
}

// ---------- 自然排序模块 ----------
mod natord {
    use std::cmp::Ordering;
    pub fn compare(a: &str, b: &str) -> Ordering {
        let mut a_chars = a.chars().peekable();
        let mut b_chars = b.chars().peekable();
        loop {
            match (a_chars.peek(), b_chars.peek()) {
                (None, None) => return Ordering::Equal,
                (None, _) => return Ordering::Less,
                (_, None) => return Ordering::Greater,
                (Some(&ac), Some(&bc)) => {
                    if ac.is_ascii_digit() && bc.is_ascii_digit() {
                        let a_num: String = a_chars.by_ref().take_while(|c| c.is_ascii_digit()).collect();
                        let b_num: String = b_chars.by_ref().take_while(|c| c.is_ascii_digit()).collect();
                        let a_val: u64 = a_num.parse().unwrap_or(u64::MAX);
                        let b_val: u64 = b_num.parse().unwrap_or(u64::MAX);
                        match a_val.cmp(&b_val) {
                            Ordering::Equal => continue,
                            other => return other,
                        }
                    } else {
                        match ac.cmp(&bc) {
                            Ordering::Equal => {
                                a_chars.next();
                                b_chars.next();
                            }
                            other => return other,
                        }
                    }
                }
            }
        }
    }
}