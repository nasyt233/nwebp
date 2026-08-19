use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use crate::scanner::{Album, AlbumImages, Config};
use chrono::DateTime;

const DEFAULT_COVER: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='300'%3E%3Crect fill='%23e0e0e0' width='200' height='300'/%3E%3Ctext fill='%23999' x='100' y='150' text-anchor='middle'%3E无封面%3C/text%3E%3C/svg%3E";

pub fn render_index(albums: &[Album], config: &Config) -> String {
    let accent_color = &config.primary_color;
    let columns = config.columns.max(1).min(6);

    let album_cards: String = albums.iter().map(|album| {
        let cover_url = album.cover.as_ref()
            .map(|c| format!("/raw?path={}", url_encode(c)))
            .unwrap_or_else(|| DEFAULT_COVER.to_string());
        let viewer_url = format!("/viewer?path={}", url_encode(&album.path));

        let time_str = if album.modified > 0 {
            if let Some(dt) = DateTime::from_timestamp(album.modified as i64, 0) {
                dt.format("%Y-%m-%d %H:%M").to_string()
            } else {
                "未知".to_string()
            }
        } else {
            "未知".to_string()
        };

        format!(r#"
        <a class="album-card" href="{viewer_url}" data-name="{name_lower}" data-modified="{modified}" data-cover="{cover}">
            <div class="cover">
                <img src="{cover}" alt="{name}" loading="lazy" decoding="async">
            </div>
            <div class="info">
                <div class="title" title="{name}">{name}</div>
                <div class="count">{count} 页</div>
                <div class="time">{time}</div>
            </div>
        </a>
        "#,
            viewer_url = viewer_url,
            cover = cover_url,
            name = html_escape(&album.name),
            name_lower = html_escape(&album.name.to_lowercase()),
            count = album.image_count,
            modified = album.modified,
            time = html_escape(&time_str),
        )
    }).collect();

    let empty_hint = if albums.is_empty() {
        r#"<div class="empty">📭 当前目录下没有找到包含图片的子目录<br><small>将漫画放在子文件夹中，每个文件夹为一本</small></div>"#
    } else {
        ""
    };

    let sort_html = r#"
<div class="sort-container">
    <label for="sortSelect">排序：</label>
    <select id="sortSelect" onchange="sortAlbums()">
        <option value="name">按名称</option>
        <option value="name-desc">按名称（倒序）</option>
        <option value="time">按时间</option>
        <option value="time-desc">按时间（倒序）</option>
        <option value="count">按数量</option>
        <option value="count-desc">按数量（倒序）</option>
    </select>
</div>
"#;

    let sort_css = r#"
.sort-container {
    margin-bottom: 15px;
    display: flex;
    align-items: center;
    gap: 8px;
}
.sort-container select {
    padding: 4px 8px;
    border-radius: 4px;
    border: 1px solid var(--border-color);
    background: var(--card-bg);
    color: var(--text-primary);
}
.album-card .time {
    font-size: 0.7em;
    color: var(--text-secondary);
    margin-top: 2px;
}
"#;

    let sort_js = r#"
function sortAlbums() {
    const grid = document.getElementById('albumGrid');
    const cards = Array.from(grid.querySelectorAll('.album-card'));
    const sortBy = document.getElementById('sortSelect').value;
    
    cards.sort((a, b) => {
        let aVal, bVal;
        if (sortBy === 'name') {
            aVal = a.getAttribute('data-name') || '';
            bVal = b.getAttribute('data-name') || '';
            return aVal.localeCompare(bVal);
        } else if (sortBy === 'name-desc') {
            aVal = a.getAttribute('data-name') || '';
            bVal = b.getAttribute('data-name') || '';
            return bVal.localeCompare(aVal);
        } else if (sortBy === 'time') {
            aVal = parseInt(a.getAttribute('data-modified')) || 0;
            bVal = parseInt(b.getAttribute('data-modified')) || 0;
            return aVal - bVal;
        } else if (sortBy === 'time-desc') {
            aVal = parseInt(a.getAttribute('data-modified')) || 0;
            bVal = parseInt(b.getAttribute('data-modified')) || 0;
            return bVal - aVal;
        } else if (sortBy === 'count') {
            const aCount = a.querySelector('.count')?.textContent || '0';
            const bCount = b.querySelector('.count')?.textContent || '0';
            aVal = parseInt(aCount) || 0;
            bVal = parseInt(bCount) || 0;
            return aVal - bVal;
        } else if (sortBy === 'count-desc') {
            const aCount = a.querySelector('.count')?.textContent || '0';
            const bCount = b.querySelector('.count')?.textContent || '0';
            aVal = parseInt(aCount) || 0;
            bVal = parseInt(bCount) || 0;
            return bVal - aVal;
        }
        return 0;
    });
    
    cards.forEach(card => grid.appendChild(card));
}
"#;

    let cover_js = r#"
const COVER_KEY = 'nwebp_show_cover';
let showCover = localStorage.getItem(COVER_KEY) === 'true';

function applyCoverState() {
    const cards = document.querySelectorAll('.album-card');
    const defaultCover = 'data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 width=%22200%22 height=%22300%22%3E%3Crect fill=%22%23e0e0e0%22 width=%22200%22 height=%22300%22/%3E%3Ctext fill=%22%23999%22 x=%22100%22 y=%22150%22 text-anchor=%22middle%22%3E无封面%3C/text%3E%3C/svg%3E';
    cards.forEach(card => {
        const img = card.querySelector('.cover img');
        if (img) {
            if (showCover) {
                img.src = card.getAttribute('data-cover') || defaultCover;
            } else {
                img.src = defaultCover;
            }
        }
    });
    const btn = document.getElementById('coverToggle');
    if (btn) btn.textContent = showCover ? '隐藏封面' : '显示封面';
}

function toggleCover() {
    showCover = !showCover;
    localStorage.setItem(COVER_KEY, showCover ? 'true' : 'false');
    applyCoverState();
}
"#;

    let popup_css = if config.popup_enabled {
        r#"
.popup-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: none;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    animation: fadeIn 0.3s ease;
}
.popup-container {
    background: var(--container-bg);
    border-radius: 16px;
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    animation: slideUp 0.3s ease;
    overflow: hidden;
    border: 1px solid var(--border-color);
}
.popup-header {
    padding: 20px 24px 16px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
}
.popup-header h2 {
    font-size: 1.3rem;
    color: var(--accent);
    margin: 0;
    font-weight: 600;
}
.popup-close {
    background: none;
    border: none;
    font-size: 28px;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0 8px;
    line-height: 1;
    transition: color 0.2s;
}
.popup-close:hover {
    color: var(--text-primary);
}
.popup-body {
    padding: 24px;
    color: var(--text-primary);
    line-height: 1.8;
    max-height: 50vh;
    overflow-y: auto;
}
.popup-body p {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
}
.popup-footer {
    padding: 16px 24px 20px;
    border-top: 1px solid var(--border-color);
    display: flex;
    justify-content: flex-end;
    gap: 12px;
}
.popup-btn {
    padding: 8px 24px;
    border: none;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
}
.popup-btn.confirm {
    background: var(--accent);
    color: #fff;
}
.popup-btn.confirm:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}
.popup-btn.cancel {
    background: var(--card-bg);
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
}
.popup-btn.cancel:hover {
    background: var(--border-color);
}
@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}
@keyframes slideUp {
    from { 
        opacity: 0;
        transform: translateY(30px) scale(0.95);
    }
    to { 
        opacity: 1;
        transform: translateY(0) scale(1);
    }
}
[data-theme="night"] .popup-overlay {
    background: rgba(0, 0, 0, 0.75);
}
[data-theme="night"] .popup-btn.cancel {
    background: var(--card-bg);
    color: var(--text-secondary);
    border-color: var(--border-color);
}
"#
    } else {
        ""
    };

    let popup_html = if config.popup_enabled {
        format!(r#"
<div id="popupOverlay" class="popup-overlay">
    <div class="popup-container">
        <div class="popup-header">
            <h2>{title}</h2>
            <button class="popup-close" onclick="closePopup()">&times;</button>
        </div>
        <div class="popup-body">
            <p>{content}</p>
        </div>
        <div class="popup-footer">
            {cancel_btn}
            <button class="popup-btn confirm" onclick="closePopup()">{confirm_text}</button>
        </div>
    </div>
</div>
"#,
            title = html_escape(&config.popup_title),
            content = html_escape(&config.popup_content).replace("\n", "<br>"),
            confirm_text = html_escape(&config.popup_confirm_text),
            cancel_btn = if !config.popup_cancel_text.is_empty() {
                format!(r#"<button class="popup-btn cancel" onclick="closePopup()">{}</button>"#, html_escape(&config.popup_cancel_text))
            } else {
                "".to_string()
            },
        )
    } else {
        "".to_string()
    };

    let popup_js = if config.popup_enabled {
        format!(r#"
const POPUP_KEY = 'nwebp_popup_shown';

function showPopup() {{
    const overlay = document.getElementById('popupOverlay');
    if (!overlay) return;
    const showOnce = {};
    if (showOnce && localStorage.getItem(POPUP_KEY) === 'true') {{
        return;
    }}
    overlay.style.display = 'flex';
    document.body.style.overflow = 'hidden';
}}

function closePopup() {{
    const overlay = document.getElementById('popupOverlay');
    if (!overlay) return;
    overlay.style.display = 'none';
    document.body.style.overflow = '';
    const showOnce = {};
    if (showOnce) {{
        localStorage.setItem(POPUP_KEY, 'true');
    }}
}}

document.addEventListener('click', function(e) {{
    const overlay = document.getElementById('popupOverlay');
    if (!overlay) return;
    if (e.target === overlay) {{
        closePopup();
    }}
}});

document.addEventListener('keydown', function(e) {{
    if (e.key === 'Escape') {{
        closePopup();
    }}
}});

if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', showPopup);
}} else {{
    showPopup();
}}
"#,
            if config.popup_show_once { "true" } else { "false" },
            if config.popup_show_once { "true" } else { "false" },
        )
    } else {
        "".to_string()
    };

    // 修复网格溢出：限制卡片宽度，居中对齐
    let grid_style = format!("grid-template-columns: repeat({}, 1fr);", columns);

    format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
:root {{
    --bg: #667eea;
    --container-bg: #ffffff;
    --text-primary: #333;
    --text-secondary: #666;
    --card-bg: #f8f8f8;
    --card-shadow: rgba(0,0,0,0.1);
    --accent: {accent};
    --search-bg: #f0f0f0;
    --cover-bg: #e0e0e0;
    --stats-bg: #f0f0f0;
    --border-color: #ddd;
}}
[data-theme="night"] {{
    --bg: #1a1a2e;
    --container-bg: #2a2a40;
    --text-primary: #eee;
    --text-secondary: #ccc;
    --card-bg: #3a3a50;
    --card-shadow: rgba(0,0,0,0.3);
    --accent: #7ab0e0;
    --search-bg: #3a3a50;
    --cover-bg: #2a2a40;
    --stats-bg: #3a3a50;
    --border-color: #555;
}}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{
    min-height: 100vh;
    font-family: 'Segoe UI', Arial, sans-serif;
    background: var(--bg);
    color: var(--text-primary);
    transition: background 0.1s, color 0.1s;
}}
.container {{
    max-width: 1400px;
    margin: 30px auto;
    padding: 30px;
    background: var(--container-bg);
    border-radius: 16px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
    border: 1px solid var(--border-color);
    transition: background 0.1s, border-color 0.1s;
}}
.header {{
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 15px;
    margin-bottom: 20px;
    padding-bottom: 15px;
    border-bottom: 1px solid var(--border-color);
}}
.header-left h1 {{
    font-size: 28px;
    font-weight: 700;
    color: var(--accent);
}}
.header-right {{
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
}}
.theme-btn {{
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 20px;
    padding: 6px 14px;
    cursor: pointer;
    font-size: 14px;
    color: var(--text-primary);
    transition: none;
}}
.search-box {{
    display: flex;
    align-items: center;
    background: var(--search-bg);
    border: 1px solid var(--border-color);
    border-radius: 20px;
    padding: 4px 14px;
}}
.search-box input {{
    border: none;
    outline: none;
    background: transparent;
    font-size: 14px;
    width: 180px;
    color: var(--text-primary);
}}
.stats {{
    display: flex;
    gap: 15px;
    flex-wrap: wrap;
    margin-bottom: 20px;
}}
.stats-item {{
    font-size: 14px;
    color: var(--text-secondary);
    background: var(--stats-bg);
    padding: 6px 16px;
    border-radius: 16px;
    border: 1px solid var(--border-color);
}}
.stats-item .num {{ font-weight: 700; color: var(--accent); }}
.subtitle {{
    color: var(--text-secondary);
    margin-bottom: 15px;
    font-size: 1em;
}}
.album-grid {{
    display: grid;
    {grid_style}
    gap: 16px;
    max-width: 100%;
    justify-items: center;
}}
.album-card {{
    text-decoration: none;
    color: inherit;
    background: var(--card-bg);
    border-radius: 10px;
    overflow: hidden;
    border: 1px solid var(--border-color);
    transition: transform 0.15s, box-shadow 0.15s;
    cursor: pointer;
    will-change: transform;
    max-width: 200px;
    width: 100%;
}}
.album-card:hover {{
    transform: translateY(-3px);
    box-shadow: 0 6px 20px var(--card-shadow);
}}
.album-card.hidden {{ display: none; }}
.cover {{
    width: 100%;
    aspect-ratio: 2/3;
    overflow: hidden;
    background: var(--cover-bg);
}}
.cover img {{
    width: 100%;
    height: 100%;
    object-fit: cover;
}}
.info {{ padding: 10px; }}
.title {{
    font-size: 0.9em;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
}}
.count {{
    font-size: 0.75em;
    color: var(--accent);
}}
.empty, .no-result {{
    text-align: center;
    padding: 40px;
    color: var(--text-secondary);
}}
.footer {{
    margin-top: 30px;
    padding-top: 20px;
    border-top: 1px solid var(--border-color);
    text-align: center;
    font-size: 13px;
    color: var(--text-secondary);
}}
.footer span {{ color: var(--accent); }}
{sort_css}
{popup_css}
@media (max-width: 768px) {{
    .container {{ margin: 15px; padding: 20px; }}
    .header-left h1 {{ font-size: 22px; }}
    .search-box input {{ width: 120px; }}
    .album-grid {{ grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); }}
}}
</style>
</head>
<body>
<div class="container">
    <div class="header">
        <div class="header-left">
            <h1>{title}</h1>
        </div>
        <div class="header-right">
            <div class="search-box">
                <span>🔍</span>
                <input type="text" id="searchInput" placeholder="搜索漫画..." oninput="filterAlbums()">
            </div>
            <button class="theme-btn" id="coverToggle" onclick="toggleCover()">显示封面</button>
            <button class="theme-btn" id="themeToggle" onclick="toggleTheme()">🌙 夜间</button>
        </div>
    </div>
    <div class="stats">
        <span class="stats-item">📚 共 <span class="num" id="totalCount">{count}</span> 本</span>
        <span class="stats-item">🔍 显示 <span class="num" id="visibleCount">{count}</span> 本</span>
    </div>
    <div class="subtitle">{subtitle}</div>
    {sort_html}
    {empty_hint}
    <div class="album-grid" id="albumGrid">
        {cards}
    </div>
    <div class="no-result" id="noResult">😢 没有找到匹配的漫画</div>
    <div class="footer">
        <p>{footer}</p>
    </div>
</div>

{popup_html}

<script>
(function() {{
    const saved = localStorage.getItem('nwebp_theme');
    if (saved === 'night') {{
        document.body.dataset.theme = 'night';
        document.getElementById('themeToggle').textContent = '☀️ 日间';
    }}
    window.toggleTheme = function() {{
        const isNight = document.body.dataset.theme === 'night';
        const newTheme = isNight ? 'day' : 'night';
        document.body.dataset.theme = newTheme;
        localStorage.setItem('nwebp_theme', newTheme);
        document.getElementById('themeToggle').textContent = newTheme === 'night' ? '☀️ 日间' : '🌙 夜间';
    }};

    window.filterAlbums = function() {{
        const query = document.getElementById('searchInput').value.toLowerCase().trim();
        const cards = document.querySelectorAll('.album-card');
        let visible = 0;
        cards.forEach(card => {{
            const name = card.getAttribute('data-name') || '';
            if (query === '' || name.includes(query)) {{
                card.classList.remove('hidden');
                visible++;
            }} else {{
                card.classList.add('hidden');
            }}
        }});
        document.getElementById('visibleCount').textContent = visible;
        document.getElementById('noResult').style.display = visible === 0 ? 'block' : 'none';
        sortAlbums();
    }};

    window.addEventListener('pageshow', (e) => {{
        if (e.persisted) {{
            window.dispatchEvent(new Event('scroll'));
        }}
    }});
}})();

{sort_js}
{cover_js}
{popup_js}

document.addEventListener('DOMContentLoaded', function() {{
    applyCoverState();
    sortAlbums();
}});
</script>
</body>
</html>
"#,
        title = config.title,
        subtitle = config.subtitle,
        footer = config.footer,
        accent = accent_color,
        count = albums.len(),
        empty_hint = empty_hint,
        cards = album_cards,
        sort_html = sort_html,
        sort_css = sort_css,
        sort_js = sort_js,
        cover_js = cover_js,
        popup_css = popup_css,
        popup_html = popup_html,
        popup_js = popup_js,
        grid_style = grid_style,
    )
}

/// 渲染阅读页
pub fn render_viewer(album: &AlbumImages) -> String {
    let images_json = serde_json::to_string(&album.images).unwrap_or_default();
    let name = html_escape(&album.name);
    let total = album.images.len();

    format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
<title>{name} - nwebp</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{
    background: #000;
    color: #eee;
    font-family: 'Segoe UI', Arial, sans-serif;
    overflow: hidden;
    height: 100vh;
}}
.toolbar {{
    position: fixed;
    top: 0; left: 0; right: 0;
    background: rgba(0,0,0,0.8);
    padding: 12px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    z-index: 100;
    transition: opacity 0.3s;
}}
.toolbar.hidden {{ opacity: 0; pointer-events: none; }}
.toolbar .title {{
    font-size: 1em;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 50%;
    color: #7ab0e0;
}}
.toolbar .controls {{
    display: flex;
    gap: 10px;
    align-items: center;
}}
.toolbar button, .toolbar a {{
    background: rgba(255,255,255,0.1);
    border: 1px solid rgba(255,255,255,0.2);
    color: #fff;
    padding: 6px 14px;
    border-radius: 20px;
    cursor: pointer;
    font-size: 0.85em;
    text-decoration: none;
    transition: background 0.2s;
}}
.toolbar button:hover, .toolbar a:hover {{
    background: rgba(255,255,255,0.25);
}}
.toolbar button.active {{
    background: #667eea;
    border-color: #667eea;
}}
.viewer-flip {{
    width: 100%;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
}}
.viewer-flip img {{
    max-width: 100%;
    max-height: 100vh;
    object-fit: contain;
    user-select: none;
    -webkit-user-drag: none;
    transition: opacity 0.2s;
}}
.viewer-scroll {{
    width: 100%;
    height: 100vh;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px 0;
}}
.viewer-scroll .lazy-placeholder {{
    width: 100%;
    max-width: 100%;
    min-height: 100px;
    background: #1a1a1a;
    margin-bottom: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
    font-size: 0.9em;
}}
.viewer-scroll img {{
    width: 100%;
    max-width: 100%;
    height: auto;
    display: block;
    margin-bottom: 10px;
}}
.nav-btn {{
    position: fixed;
    top: 50%;
    transform: translateY(-50%);
    background: rgba(0,0,0,0.5);
    border: 1px solid rgba(255,255,255,0.2);
    color: #fff;
    width: 50px;
    height: 80px;
    font-size: 1.8em;
    cursor: pointer;
    z-index: 50;
    border-radius: 10px;
    transition: background 0.2s;
}}
.nav-btn:hover {{ background: rgba(255,255,255,0.2); }}
.nav-btn:disabled {{ opacity: 0.2; cursor: default; }}
.nav-prev {{ left: 10px; }}
.nav-next {{ right: 10px; }}
.page-indicator {{
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(0,0,0,0.7);
    padding: 6px 16px;
    border-radius: 20px;
    font-size: 0.9em;
    z-index: 100;
    transition: opacity 0.3s;
    border: 1px solid rgba(255,255,255,0.1);
}}
.page-indicator.hidden {{ opacity: 0; }}
.loading {{
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    color: #888;
    font-size: 1.2em;
}}
</style>
</head>
<body>
<div class="toolbar" id="toolbar">
    <a href="/">← 返回</a>
    <div class="title">{name}</div>
    <div class="controls">
        <button id="modeBtn" onclick="toggleMode()" class="active">翻页模式</button>
    </div>
</div>
<div id="viewer" class="viewer-flip">
    <img id="currentImg" src="" alt="">
</div>
<button class="nav-btn nav-prev" id="prevBtn" onclick="prevPage()">‹</button>
<button class="nav-btn nav-next" id="nextBtn" onclick="nextPage()">›</button>
<div class="page-indicator" id="pageIndicator">1 / {total}</div>
<div class="loading" id="loading">加载中...</div>
<script>
const images = {images_json};
const total = images.length;
let current = 0;
let flipMode = true;
let preloadCache = new Map();
let scrollObserver = null;

function imgUrl(path) {{ return '/raw?path=' + encodeURIComponent(path); }}
function getImg() {{ return document.getElementById('currentImg'); }}

function preload(index) {{
    if (index < 0 || index >= total || preloadCache.has(index)) return;
    const img = new Image();
    img.src = imgUrl(images[index]);
    preloadCache.set(index, img);
}}

function showPage(index) {{
    if (index < 0 || index >= total) return;
    current = index;
    const imgEl = getImg();
    if (!imgEl) return;
    loading.style.display = 'block';
    imgEl.style.opacity = '0';
    const img = new Image();
    img.onload = () => {{
        imgEl.src = img.src;
        imgEl.style.opacity = '1';
        loading.style.display = 'none';
    }};
    img.onerror = () => {{ loading.textContent = '加载失败'; }};
    img.src = imgUrl(images[index]);
    for (let i = index - 5; i <= index + 5; i++) {{
        if (i !== index) preload(i);
    }}
    updateUI();
}}

function updateUI() {{
    pageIndicator.textContent = (current + 1) + ' / ' + total;
    prevBtn.disabled = current === 0;
    nextBtn.disabled = current === total - 1;
}}

function prevPage() {{ if (current > 0) showPage(current - 1); }}
function nextPage() {{ if (current < total - 1) showPage(current + 1); }}

function toggleMode() {{
    flipMode = !flipMode;
    const viewer = document.getElementById('viewer');
    const modeBtn = document.getElementById('modeBtn');
    if (flipMode) {{
        modeBtn.textContent = '翻页模式';
        modeBtn.classList.add('active');
        viewer.className = 'viewer-flip';
        viewer.innerHTML = '<img id="currentImg" src="" alt="" style="transition:opacity 0.2s;">';
        prevBtn.style.display = 'block';
        nextBtn.style.display = 'block';
        pageIndicator.style.display = 'block';
        if (scrollObserver) {{
            scrollObserver.disconnect();
            scrollObserver = null;
        }}
        showPage(current);
    }} else {{
        modeBtn.textContent = '滚动模式';
        modeBtn.classList.remove('active');
        viewer.className = 'viewer-scroll';
        viewer.innerHTML = '';
        prevBtn.style.display = 'none';
        nextBtn.style.display = 'none';
        pageIndicator.style.display = 'none';
        setupScrollMode();
    }}
}}

function setupScrollMode() {{
    const viewer = document.getElementById('viewer');
    const fragment = document.createDocumentFragment();
    for (let i = 0; i < total; i++) {{
        const container = document.createElement('div');
        container.className = 'lazy-placeholder';
        container.dataset.index = i;
        container.textContent = '加载中...';
        fragment.appendChild(container);
    }}
    viewer.appendChild(fragment);

    const observer = new IntersectionObserver((entries) => {{
        entries.forEach(entry => {{
            if (entry.isIntersecting) {{
                const container = entry.target;
                const index = parseInt(container.dataset.index, 10);
                if (!container.dataset.loaded) {{
                    container.dataset.loaded = 'true';
                    const img = new Image();
                    img.onload = () => {{
                        container.innerHTML = '';
                        container.style.display = 'block';
                        container.style.minHeight = 'auto';
                        container.style.height = 'auto';
                        img.style.width = '100%';
                        img.style.height = 'auto';
                        img.style.display = 'block';
                        container.appendChild(img);
                        preload(index - 1);
                        preload(index + 1);
                    }};
                    img.onerror = () => {{
                        container.textContent = '加载失败';
                        container.style.minHeight = '100px';
                    }};
                    img.src = imgUrl(images[index]);
                }}
                observer.unobserve(container);
            }}
        }});
    }}, {{ rootMargin: '300px' }});

    document.querySelectorAll('.lazy-placeholder').forEach(el => observer.observe(el));
    scrollObserver = observer;
}}

document.addEventListener('keydown', (e) => {{
    if (!flipMode) return;
    if (e.key === 'ArrowLeft' || e.key === 'a' || e.key === 'A') prevPage();
    else if (e.key === 'ArrowRight' || e.key === 'd' || e.key === 'D' || e.key === ' ') {{ e.preventDefault(); nextPage(); }}
    else if (e.key === 'Home') showPage(0);
    else if (e.key === 'End') showPage(total - 1);
}});

let touchStartX = 0, touchStartY = 0;
document.addEventListener('touchstart', (e) => {{
    touchStartX = e.touches[0].clientX;
    touchStartY = e.touches[0].clientY;
}}, {{ passive: true }});
document.addEventListener('touchend', (e) => {{
    if (!flipMode) return;
    const dx = e.changedTouches[0].clientX - touchStartX;
    const dy = e.changedTouches[0].clientY - touchStartY;
    if (Math.abs(dx) > 50 && Math.abs(dx) > Math.abs(dy)) {{
        if (dx > 0) prevPage(); else nextPage();
    }}
}}, {{ passive: true }});

document.getElementById('viewer').addEventListener('click', (e) => {{
    if (!flipMode) return;
    const w = window.innerWidth, x = e.clientX;
    if (x < w * 0.3) prevPage();
    else if (x > w * 0.7) nextPage();
}});

let toolbarTimer;
document.addEventListener('click', () => {{
    toolbar.classList.remove('hidden');
    pageIndicator.classList.remove('hidden');
    clearTimeout(toolbarTimer);
    toolbarTimer = setTimeout(() => {{
        toolbar.classList.add('hidden');
        if (flipMode) pageIndicator.classList.add('hidden');
    }}, 3000);
}});

if (total > 0) showPage(0);
else loading.textContent = '没有图片';
</script>
</body>
</html>
"#,
        name = name,
        total = total,
        images_json = images_json,
    )
}

fn url_encode(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}