use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use walkdir::WalkDir;

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];
const CACHE_TTL: Duration = Duration::from_secs(5);

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)] pub title: String,
    #[serde(default)] pub subtitle: String,
    #[serde(default)] pub primary_color: String,
    #[serde(default)] pub footer: String,
    #[serde(default)] pub popup_enabled: bool,
    #[serde(default)] pub popup_title: String,
    #[serde(default)] pub popup_content: String,
    #[serde(default)] pub popup_confirm_text: String,
    #[serde(default)] pub popup_cancel_text: String,
    #[serde(default)] pub popup_show_once: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "📚 nwebp 漫画库".to_string(),
            subtitle: "轻量级网页漫画阅读器".to_string(),
            primary_color: "#667eea".to_string(),
            footer: "⚡ 由 Rust 强力驱动 · nwebp 漫画浏览".to_string(),
            popup_enabled: false,
            popup_title: "📢 公告".to_string(),
            popup_content: "欢迎访问 nwebp 漫画库！".to_string(),
            popup_confirm_text: "知道了".to_string(),
            popup_cancel_text: "".to_string(),
            popup_show_once: true,
        }
    }
}

impl Config {
    pub fn load_from_file(root: &Path) -> Self {
        let path = root.join("nwebp.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&content) { return cfg; }
        }
        let cfg = Config::default();
        let _ = std::fs::write(&path, serde_json::to_string_pretty(&cfg).unwrap());
        cfg
    }
}

pub struct AppState {
    pub root_dir: PathBuf,
    pub config: Arc<RwLock<Config>>,
    views: Arc<RwLock<HashMap<String, u64>>>,
    views_path: PathBuf,
    albums_cache: Arc<Mutex<Option<(Vec<Album>, Instant)>>>,
    images_cache: Arc<Mutex<HashMap<String, (AlbumImages, Instant)>>>,
}

impl AppState {
    pub fn new(root_dir: PathBuf) -> Self {
        let views_path = root_dir.join("nwebp_views.json");
        let views: HashMap<String, u64> = std::fs::read_to_string(&views_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default();
        let config = Arc::new(RwLock::new(Config::load_from_file(&root_dir)));
        Self {
            root_dir,
            config,
            views: Arc::new(RwLock::new(views)),
            views_path,
            albums_cache: Arc::new(Mutex::new(None)),
            images_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_config(&self) -> Config { self.config.read().await.clone() }

    pub async fn increment_view(&self, path: &str) {
        let snapshot = {
            let mut views = self.views.write().await;
            let c = views.entry(path.to_string()).or_insert(0);
            *c += 1;
            views.clone()
        };
        let vp = self.views_path.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
                let _ = std::fs::write(&vp, json);
            }
        }).await.ok();
    }

    pub async fn get_all_views(&self) -> HashMap<String, u64> {
        self.views.read().await.clone()
    }

    pub async fn scan_albums(&self) -> Vec<Album> {
        {
            let cache = self.albums_cache.lock().await;
            if let Some((a, t)) = cache.as_ref() {
                if t.elapsed() < CACHE_TTL { return a.clone(); }
            }
        }
        let root_dir = self.root_dir.clone();
        let albums = tokio::task::spawn_blocking(move || do_scan_albums(&root_dir)).await.unwrap();
        let mut cache = self.albums_cache.lock().await;
        *cache = Some((albums.clone(), Instant::now()));
        albums
    }

    pub async fn scan_folder(&self, rel_path: &str) -> BrowseResult {
        let root_dir = self.root_dir.clone();
        let path = rel_path.to_string();
        tokio::task::spawn_blocking(move || do_scan_folder(&root_dir, &path)).await.unwrap()
    }

    pub async fn get_album_images(&self, album_path: &str) -> Option<AlbumImages> {
        {
            let cache = self.images_cache.lock().await;
            if let Some((i, t)) = cache.get(album_path) {
                if t.elapsed() < CACHE_TTL { return Some(i.clone()); }
            }
        }
        let root_dir = self.root_dir.clone();
        let path = album_path.to_string();
        let result = tokio::task::spawn_blocking(move || do_get_album_images(&root_dir, &path)).await.unwrap();
        if let Some(images) = result {
            let mut cache = self.images_cache.lock().await;
            cache.insert(album_path.to_string(), (images.clone(), Instant::now()));
            Some(images)
        } else { None }
    }

    pub fn resolve_path(&self, rel_path: &str) -> Option<PathBuf> {
        if rel_path.is_empty() { return None; }
        let full = self.root_dir.join(rel_path);
        if let Ok(canon) = full.canonicalize() {
            if canon.starts_with(&self.root_dir) && canon.is_file() { return Some(canon); }
        }
        let normalized = full.components().collect::<PathBuf>();
        if normalized.starts_with(&self.root_dir) && normalized.is_file() { Some(normalized) } else { None }
    }
}

fn get_modified_time(path: &Path) -> u64 {
    if let Ok(metadata) = std::fs::metadata(path) {
        if let Ok(time) = metadata.modified() {
            if let Ok(d) = time.duration_since(UNIX_EPOCH) { return d.as_secs(); }
        }
    }
    0
}

fn list_images_with_size(dir: &Path) -> (Vec<PathBuf>, u64) {
    let mut images = Vec::new();
    let mut total: u64 = 0;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() && is_image(&p) {
                if let Ok(m) = p.metadata() { total += m.len(); }
                images.push(p);
            }
        }
    }
    (images, total)
}

fn do_scan_albums(root_dir: &Path) -> Vec<Album> {
    let mut albums = Vec::new();
    for entry in WalkDir::new(root_dir).min_depth(1).max_depth(3) {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        if !entry.file_type().is_dir() { continue; }
        let path = entry.path();
        let (mut images, size) = list_images_with_size(path);
        if images.is_empty() { continue; }
        images.sort_by(|a, b| {
            let an = a.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            let bn = b.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            natord::compare(&an, &bn)
        });
        let relative = path.strip_prefix(root_dir).unwrap_or(path).to_string_lossy().to_string();
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| relative.clone());
        let cover = images.first().map(|p| p.strip_prefix(root_dir).unwrap_or(p).to_string_lossy().to_string());
        albums.push(Album { name, path: relative, image_count: images.len(), cover, modified: get_modified_time(path), size });
    }
    albums.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    albums
}

fn do_scan_folder(root_dir: &Path, rel_path: &str) -> BrowseResult {
    let full_path = if rel_path.is_empty() { root_dir.to_path_buf() } else { root_dir.join(rel_path) };
    let mut folders = Vec::new();
    let mut albums = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&full_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if name.starts_with('.') { continue; }
            let rel = path.strip_prefix(root_dir).unwrap_or(&path).to_string_lossy().to_string();
            let (mut images, size) = list_images_with_size(&path);
            let modified = get_modified_time(&path);
            if !images.is_empty() {
                images.sort_by(|a, b| {
                    let an = a.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                    let bn = b.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                    natord::compare(&an, &bn)
                });
                let cover = images.first().map(|p| p.strip_prefix(root_dir).unwrap_or(p).to_string_lossy().to_string());
                albums.push(Album { name, path: rel, image_count: images.len(), cover, modified, size });
            } else {
                let has_sub = std::fs::read_dir(&path).map(|rd| rd.flatten().any(|e| e.path().is_dir())).unwrap_or(false);
                if has_sub {
                    let cnt = std::fs::read_dir(&path).map(|rd| rd.flatten().filter(|e| e.path().is_dir()).count()).unwrap_or(0);
                    folders.push(Folder { name, path: rel, modified, child_count: cnt });
                }
            }
        }
    }
    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    albums.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    let mut breadcrumbs = vec![Breadcrumb { name: "🏠 主页".to_string(), path: String::new() }];
    if !rel_path.is_empty() {
        let parts: Vec<&str> = rel_path.split('/').collect();
        let mut acc = String::new();
        for part in &parts {
            if !acc.is_empty() { acc.push('/'); }
            acc.push_str(part);
            breadcrumbs.push(Breadcrumb { name: part.to_string(), path: acc.clone() });
        }
    }
    BrowseResult { folders, albums, current_path: rel_path.to_string(), breadcrumbs }
}

fn do_get_album_images(root_dir: &Path, album_path: &str) -> Option<AlbumImages> {
    let full_path = root_dir.join(album_path);
    if !full_path.is_dir() { return None; }
    let (mut images, _) = list_images_with_size(&full_path);
    images.sort_by(|a, b| {
        let an = a.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        let bn = b.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        natord::compare(&an, &bn)
    });
    let rel: Vec<String> = images.iter().map(|p| p.strip_prefix(root_dir).unwrap_or(p).to_string_lossy().to_string()).collect();
    let name = full_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| album_path.to_string());
    Some(AlbumImages { name, path: album_path.to_string(), images: rel })
}

#[derive(Serialize, Clone)] pub struct Album { pub name: String, pub path: String, pub image_count: usize, pub cover: Option<String>, pub modified: u64, pub size: u64 }
#[derive(Serialize, Clone)] pub struct AlbumImages { pub name: String, pub path: String, pub images: Vec<String> }
#[derive(Serialize, Clone)] pub struct Folder { pub name: String, pub path: String, pub modified: u64, pub child_count: usize }
#[derive(Serialize, Clone)] pub struct Breadcrumb { pub name: String, pub path: String }
#[derive(Serialize, Clone)] pub struct BrowseResult { pub folders: Vec<Folder>, pub albums: Vec<Album>, pub current_path: String, pub breadcrumbs: Vec<Breadcrumb> }

fn is_image(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).map(|e| IMAGE_EXTS.contains(&e.to_lowercase().as_str())).unwrap_or(false)
}

mod natord {
    use std::cmp::Ordering;
    pub fn compare(a: &str, b: &str) -> Ordering {
        let mut ac = a.chars().peekable();
        let mut bc = b.chars().peekable();
        loop {
            match (ac.peek(), bc.peek()) {
                (None, None) => return Ordering::Equal,
                (None, _) => return Ordering::Less,
                (_, None) => return Ordering::Greater,
                (Some(&x), Some(&y)) => {
                    if x.is_ascii_digit() && y.is_ascii_digit() {
                        let an: String = ac.by_ref().take_while(|c| c.is_ascii_digit()).collect();
                        let bn: String = bc.by_ref().take_while(|c| c.is_ascii_digit()).collect();
                        let av: u64 = an.parse().unwrap_or(u64::MAX);
                        let bv: u64 = bn.parse().unwrap_or(u64::MAX);
                        match av.cmp(&bv) { Ordering::Equal => continue, o => return o }
                    } else {
                        match x.cmp(&y) { Ordering::Equal => { ac.next(); bc.next(); }, o => return o }
                    }
                }
            }
        }
    }
}