use crate::scanner::AlbumImages;
use super::shared::html_escape;

pub fn render_viewer(album: &AlbumImages) -> String {
    let images_json = serde_json::to_string(&album.images).unwrap_or_default();
    let name = html_escape(&album.name);
    let name_json = serde_json::to_string(&album.name).unwrap_or_default();
    let album_path_json = serde_json::to_string(&album.path).unwrap_or_default();
    let total = album.images.len();

    format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
<title>{name} - nwebp</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ background: #000; color: #eee; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif; overflow: hidden; height: 100vh; }}
.toolbar {{
    position: fixed; top: 0; left: 0; right: 0;
    background: rgba(0,0,0,0.85); padding: 12px 20px;
    display: flex; justify-content: space-between; align-items: center;
    z-index: 100; transition: opacity 0.3s ease, transform 0.3s ease;
    backdrop-filter: blur(10px);
}}
.toolbar.hidden {{ opacity: 0; pointer-events: none; transform: translateY(-10px); }}
.toolbar .title {{ font-size: 0.95em; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 55%; color: #7ab0e0; }}
.toolbar .controls {{ display: flex; gap: 8px; align-items: center; }}
.toolbar button, .toolbar a {{
    background: rgba(255,255,255,0.1);
    border: 1px solid rgba(255,255,255,0.15);
    color: #fff; padding: 6px 14px; border-radius: 20px;
    cursor: pointer; font-size: 0.85em; text-decoration: none;
    transition: background-color 0.2s ease;
}}
.toolbar button:hover, .toolbar a:hover {{ background: rgba(255,255,255,0.25); }}
.toolbar button.active {{ background: #667eea; border-color: #667eea; }}
.viewer-flip {{ width: 100%; height: 100vh; display: flex; align-items: center; justify-content: center; }}
.viewer-flip img {{ max-width: 100%; max-height: 100vh; object-fit: contain; user-select: none; -webkit-user-drag: none; }}
.viewer-scroll {{ width: 100%; height: 100vh; overflow-y: auto; overflow-x: hidden; display: flex; flex-direction: column; align-items: center; padding: 20px 0; }}
.viewer-scroll .lazy-placeholder {{ width: 100%; min-height: 100px; background: #1a1a1a; margin-bottom: 10px; display: flex; align-items: center; justify-content: center; color: #555; font-size: 0.9em; }}
.viewer-scroll img {{ width: 100%; max-width: 100%; height: auto; display: block; margin-bottom: 10px; }}
.nav-btn {{
    position: fixed; top: 50%; transform: translateY(-50%);
    background: rgba(0,0,0,0.5); border: 1px solid rgba(255,255,255,0.15);
    color: #fff; width: 50px; height: 80px; font-size: 1.8em;
    cursor: pointer; z-index: 50; border-radius: 10px;
    transition: background-color 0.2s ease, opacity 0.3s ease;
}}
.nav-btn:hover {{ background: rgba(255,255,255,0.2); }}
.nav-btn:disabled {{ opacity: 0.2; cursor: default; }}
.nav-btn.hidden {{ display: none; }}
.nav-prev {{ left: 10px; }}
.nav-next {{ right: 10px; }}
.bottom-bar {{
    position: fixed; bottom: 0; left: 0; right: 0;
    background: rgba(0,0,0,0.85); padding: 10px 20px;
    display: flex; justify-content: center; align-items: center; gap: 12px;
    z-index: 100; transition: opacity 0.3s ease, transform 0.3s ease;
    backdrop-filter: blur(10px); flex-wrap: wrap;
}}
.bottom-bar.hidden {{ opacity: 0; pointer-events: none; transform: translateY(10px); }}
.bottom-btn {{
    background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.15);
    color: #fff; padding: 6px 14px; border-radius: 20px;
    cursor: pointer; font-size: 0.85em;
    transition: background-color 0.2s ease;
}}
.bottom-btn:hover {{ background: rgba(255,255,255,0.25); }}
.page-info {{ display: flex; align-items: center; gap: 6px; color: #fff; font-size: 0.9em; }}
.page-input {{ width: 60px; text-align: center; background: rgba(255,255,255,0.15); border: 1px solid rgba(255,255,255,0.2); color: #fff; padding: 4px 6px; border-radius: 6px; font-size: 0.95em; font-family: inherit; }}
.page-input:focus {{ outline: none; border-color: #667eea; background: rgba(102,126,234,0.3); }}
.page-total {{ color: #aaa; }}
.loading {{ position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); color: #888; font-size: 1.2em; z-index: 90; display: none; }}
.loading.show {{ display: block; }}
.back-to-top {{
    position: fixed; bottom: 70px; right: 20px;
    width: 40px; height: 40px; border-radius: 50%;
    background: rgba(102,126,234,0.9); color: #fff; border: none;
    font-size: 18px; cursor: pointer; z-index: 150;
    box-shadow: 0 4px 16px rgba(0,0,0,0.3);
    opacity: 0; visibility: hidden; transform: translateY(10px);
    transition: all 0.3s ease;
}}
.back-to-top.show {{ opacity: 1; visibility: visible; transform: translateY(0); }}
.settings-overlay {{ position: fixed; inset: 0; background: rgba(0,0,0,0.6); backdrop-filter: blur(6px); display: none; align-items: center; justify-content: center; z-index: 200; }}
.settings-overlay.show {{ display: flex; animation: fadeIn 0.25s ease; }}
.settings-modal {{ background: #1e1e2e; color: #eee; border-radius: 14px; max-width: 400px; width: 90%; overflow: hidden; border: 1px solid #333; animation: slideUp 0.3s ease; }}
.settings-header {{ padding: 16px 20px; border-bottom: 1px solid #333; display: flex; justify-content: space-between; align-items: center; }}
.settings-header h2 {{ font-size: 1.05rem; color: #7ab0e0; margin: 0; }}
.settings-close {{ background: none; border: none; font-size: 22px; color: #999; cursor: pointer; line-height: 1; }}
.settings-body {{ padding: 16px 20px; display: flex; flex-direction: column; gap: 14px; }}
.setting-row {{ display: flex; justify-content: space-between; align-items: center; gap: 12px; }}
.setting-label {{ font-size: 13px; flex: 1; }}
.setting-label small {{ display: block; color: #999; font-size: 11px; margin-top: 2px; }}
.toggle-switch {{ position: relative; width: 40px; height: 22px; background: #444; border-radius: 11px; cursor: pointer; transition: background-color 0.25s ease; flex-shrink: 0; }}
.toggle-switch.on {{ background: #667eea; }}
.toggle-switch::after {{ content: ''; position: absolute; top: 2px; left: 2px; width: 18px; height: 18px; background: #fff; border-radius: 50%; transition: transform 0.25s ease; }}
.toggle-switch.on::after {{ transform: translateX(18px); }}
.settings-footer {{ padding: 12px 20px 16px; display: flex; justify-content: flex-end; border-top: 1px solid #333; }}
.primary-btn {{ padding: 8px 20px; border-radius: 8px; background: #667eea; color: #fff; border: none; cursor: pointer; font-size: 13px; }}
@keyframes fadeIn {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
@keyframes slideUp {{ from {{ opacity: 0; transform: translateY(20px) scale(0.96); }} to {{ opacity: 1; transform: translateY(0) scale(1); }} }}
@media (max-width: 640px) {{
    .toolbar {{ padding: 10px 14px; }}
    .toolbar .title {{ font-size: 0.85em; max-width: 45%; }}
    .toolbar button, .toolbar a {{ padding: 5px 10px; font-size: 0.8em; }}
    .bottom-bar {{ padding: 8px 12px; gap: 8px; }}
    .bottom-btn {{ padding: 5px 10px; font-size: 0.8em; }}
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
<button class="back-to-top" id="backTop" onclick="scrollToTop()" title="返回顶部">↑</button>
<div class="bottom-bar" id="bottomBar">
    <div class="page-info">
        <input type="number" class="page-input" id="pageInput" min="1" max="{total}" value="1">
        <span class="page-total">/ {total}</span>
    </div>
    <button class="bottom-btn" onclick="openViewerSettings()" title="阅读设置">⚙️ 设置</button>
</div>
<div class="loading" id="loading">加载中...</div>

<div class="settings-overlay" id="viewerSettings" onclick="if(event.target===this)closeViewerSettings()">
    <div class="settings-modal">
        <div class="settings-header">
            <h2>⚙️ 阅读设置</h2>
            <button class="settings-close" onclick="closeViewerSettings()">&times;</button>
        </div>
        <div class="settings-body">
            <div class="setting-row">
                <div class="setting-label">显示翻页按钮<small>左右箭头按钮</small></div>
                <div class="toggle-switch" id="setArrows" onclick="toggleArrows()"></div>
            </div>
            <div class="setting-row">
                <div class="setting-label">反向阅读<small>从右向左翻页</small></div>
                <div class="toggle-switch" id="setRtl" onclick="toggleRtl()"></div>
            </div>
            <div class="setting-row">
                <div class="setting-label">点击翻页<small>点击左右两侧切换</small></div>
                <div class="toggle-switch" id="setClick" onclick="toggleClick()"></div>
            </div>
            <div class="setting-row">
                <div class="setting-label">触屏滑动<small>左右滑动切换页面</small></div>
                <div class="toggle-switch" id="setTouch" onclick="toggleTouch()"></div>
            </div>
        </div>
        <div class="settings-footer">
            <button class="primary-btn" onclick="closeViewerSettings()">完成</button>
        </div>
    </div>
</div>

<script>
const images = {images_json};
const albumPath = {album_path_json};
const albumName = {name_json};
const total = images.length;
let current = 0;
let flipMode = true;
let preloadCache = new Map();
let scrollObserver = null;
let scrollLock = false;

const KEY_MODE = 'nwebp_viewer_mode';
const KEY_ARROWS = 'nwebp_viewer_arrows';
const KEY_RTL = 'nwebp_viewer_rtl';
const KEY_CLICK = 'nwebp_viewer_click';
const KEY_TOUCH = 'nwebp_viewer_touch';
const PROGRESS_KEY = 'nwebp_progress_' + albumPath;
const HISTORY_KEY = 'nwebp_history';

let showArrows = localStorage.getItem(KEY_ARROWS) !== 'false';
let reverseRead = localStorage.getItem(KEY_RTL) === 'true';
let enableClick = localStorage.getItem(KEY_CLICK) !== 'false';
let enableTouch = localStorage.getItem(KEY_TOUCH) !== 'false';

(function saveHistory() {{
    try {{
        let history = JSON.parse(localStorage.getItem(HISTORY_KEY) || '[]');
        history = history.filter(h => h.path !== albumPath);
        history.unshift({{ name: albumName, path: albumPath, time: Date.now() }});
        history = history.slice(0, 20);
        localStorage.setItem(HISTORY_KEY, JSON.stringify(history));
    }} catch (e) {{}}
}})();

function imgUrl(path) {{ return '/raw?path=' + encodeURIComponent(path); }}
function getImg() {{ return document.getElementById('currentImg'); }}

function loadProgress() {{
    try {{
        const data = JSON.parse(localStorage.getItem(PROGRESS_KEY));
        if (data && typeof data.page === 'number' && data.page >= 0 && data.page < total) return data.page;
    }} catch (e) {{}}
    return 0;
}}

function saveProgress() {{
    try {{
        localStorage.setItem(PROGRESS_KEY, JSON.stringify({{ page: current, total: total, time: Date.now() }}));
    }} catch (e) {{}}
}}

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

    loading.classList.add('show');
    imgEl.style.opacity = '0';

    const onLoad = () => {{
        imgEl.style.opacity = '1';
        loading.classList.remove('show');
        imgEl.removeEventListener('load', onLoad);
        imgEl.removeEventListener('error', onError);
    }};
    const onError = () => {{
        loading.textContent = '加载失败';
        imgEl.removeEventListener('load', onLoad);
        imgEl.removeEventListener('error', onError);
    }};
    imgEl.addEventListener('load', onLoad);
    imgEl.addEventListener('error', onError);
    imgEl.src = imgUrl(images[index]);

    for (let i = index - 5; i <= index + 5; i++) {{
        if (i !== index) preload(i);
    }}
    updateUI();
    saveProgress();
}}

function updateUI() {{
    const input = document.getElementById('pageInput');
    if (input) input.value = current + 1;
    prevBtn.disabled = current === 0;
    nextBtn.disabled = current === total - 1;
}}

function jumpToPage() {{
    const input = document.getElementById('pageInput');
    if (!input) return;
    let page = parseInt(input.value);
    if (isNaN(page) || page < 1 || page > total) {{ input.value = current + 1; return; }}
    page = page - 1;
    if (flipMode) {{ showPage(page); }}
    else {{
        const container = document.querySelector('.lazy-placeholder[data-index=' + JSON.stringify(String(page)) + ']');
        if (container) container.scrollIntoView({{ behavior: 'smooth', block: 'start' }});
    }}
}}

function prevPage() {{
    if (reverseRead) {{ if (current < total - 1) showPage(current + 1); }}
    else {{ if (current > 0) showPage(current - 1); }}
}}

function nextPage() {{
    if (reverseRead) {{ if (current > 0) showPage(current - 1); }}
    else {{ if (current < total - 1) showPage(current + 1); }}
}}

function applyMode() {{
    const viewer = document.getElementById('viewer');
    const modeBtn = document.getElementById('modeBtn');
    if (flipMode) {{
        modeBtn.textContent = '翻页模式';
        modeBtn.classList.add('active');
        viewer.className = 'viewer-flip';
        viewer.innerHTML = '<img id="currentImg" src="" alt="">';
        prevBtn.classList.toggle('hidden', !showArrows);
        nextBtn.classList.toggle('hidden', !showArrows);
        bottomBar.classList.remove('hidden');
        if (scrollObserver) {{ scrollObserver.disconnect(); scrollObserver = null; }}
        showPage(current);
    }} else {{
        modeBtn.textContent = '滚动模式';
        modeBtn.classList.remove('active');
        viewer.className = 'viewer-scroll';
        viewer.innerHTML = '';
        prevBtn.classList.add('hidden');
        nextBtn.classList.add('hidden');
        bottomBar.classList.remove('hidden');
        setupScrollMode();
    }}
    localStorage.setItem(KEY_MODE, flipMode ? 'flip' : 'scroll');
}}

function toggleMode() {{ flipMode = !flipMode; applyMode(); }}

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
    scrollLock = true;

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
                        container.style.minHeight = 'auto';
                        container.style.height = 'auto';
                        img.style.width = '100%';
                        img.style.height = 'auto';
                        img.style.display = 'block';
                        container.appendChild(img);
                        preload(index - 1); preload(index + 1);
                    }};
                    img.onerror = () => {{ container.textContent = '加载失败'; }};
                    img.src = imgUrl(images[index]);
                }}
                observer.unobserve(container);
            }}
        }});
    }}, {{ rootMargin: '300px' }});

    document.querySelectorAll('.lazy-placeholder').forEach(el => observer.observe(el));
    scrollObserver = observer;

    setTimeout(() => {{
        const c = document.querySelector('.lazy-placeholder[data-index=' + JSON.stringify(String(current)) + ']');
        if (c) c.scrollIntoView({{ block: 'start' }});
        scrollLock = false;
    }}, 150);
}}

function onViewerScroll() {{
    if (flipMode || scrollLock) return;
    const viewer = document.getElementById('viewer');
    const containers = viewer.querySelectorAll('.lazy-placeholder');
    if (containers.length === 0) return;
    const viewerTop = viewer.getBoundingClientRect().top;
    let topIndex = 0;
    for (let i = 0; i < containers.length; i++) {{
        const r = containers[i].getBoundingClientRect();
        if (r.bottom > viewerTop + 100) {{ topIndex = i; break; }}
    }}
    if (topIndex !== current) {{
        current = topIndex;
        updateUI();
        saveProgress();
    }}
}}

document.addEventListener('keydown', (e) => {{
    if (e.target.tagName === 'INPUT') return;
    if (!flipMode) return;
    if (e.key === 'ArrowLeft' || e.key === 'a' || e.key === 'A') prevPage();
    else if (e.key === 'ArrowRight' || e.key === 'd' || e.key === 'D' || e.key === ' ') {{ e.preventDefault(); nextPage(); }}
    else if (e.key === 'Home') showPage(0);
    else if (e.key === 'End') showPage(total - 1);
}});

let touchStartX = 0, touchStartY = 0;
document.addEventListener('touchstart', (e) => {{ touchStartX = e.touches[0].clientX; touchStartY = e.touches[0].clientY; }}, {{ passive: true }});
document.addEventListener('touchend', (e) => {{
    if (!flipMode || !enableTouch) return;
    const dx = e.changedTouches[0].clientX - touchStartX;
    const dy = e.changedTouches[0].clientY - touchStartY;
    if (Math.abs(dx) > 50 && Math.abs(dx) > Math.abs(dy)) {{ if (dx > 0) prevPage(); else nextPage(); }}
}}, {{ passive: true }});

document.getElementById('viewer').addEventListener('click', (e) => {{
    if (!flipMode || !enableClick) return;
    const w = window.innerWidth, x = e.clientX;
    if (x < w * 0.3) prevPage();
    else if (x > w * 0.7) nextPage();
}});

let toolbarTimer;
function showBars() {{
    toolbar.classList.remove('hidden');
    bottomBar.classList.remove('hidden');
    clearTimeout(toolbarTimer);
    if (flipMode) {{
        toolbarTimer = setTimeout(() => {{
            toolbar.classList.add('hidden');
            bottomBar.classList.add('hidden');
        }}, 3000);
    }}
}}

document.addEventListener('click', () => showBars());
document.addEventListener('mousemove', () => showBars());

document.getElementById('pageInput').addEventListener('change', jumpToPage);
document.getElementById('pageInput').addEventListener('keydown', (e) => {{
    if (e.key === 'Enter') {{ e.preventDefault(); jumpToPage(); e.target.blur(); }}
}});

function scrollToTop() {{
    if (flipMode) {{ showPage(0); }}
    else {{
        const v = document.getElementById('viewer');
        v.scrollTo({{ top: 0, behavior: 'smooth' }});
    }}
}}

function handleScrollEvent() {{
    const btn = document.getElementById('backTop');
    if (!btn) return;
    let y = 0;
    if (flipMode) y = window.scrollY;
    else {{ const v = document.getElementById('viewer'); y = v ? v.scrollTop : 0; }}
    btn.classList.toggle('show', y > 300);
    onViewerScroll();
}}

window.addEventListener('scroll', handleScrollEvent);

function openViewerSettings() {{
    document.getElementById('viewerSettings').classList.add('show');
    document.getElementById('setArrows').classList.toggle('on', showArrows);
    document.getElementById('setRtl').classList.toggle('on', reverseRead);
    document.getElementById('setClick').classList.toggle('on', enableClick);
    document.getElementById('setTouch').classList.toggle('on', enableTouch);
}}

function closeViewerSettings() {{ document.getElementById('viewerSettings').classList.remove('show'); }}

function toggleArrows() {{
    showArrows = !showArrows;
    localStorage.setItem(KEY_ARROWS, showArrows ? 'true' : 'false');
    document.getElementById('setArrows').classList.toggle('on', showArrows);
    if (flipMode) {{
        prevBtn.classList.toggle('hidden', !showArrows);
        nextBtn.classList.toggle('hidden', !showArrows);
    }}
}}

function toggleRtl() {{
    reverseRead = !reverseRead;
    localStorage.setItem(KEY_RTL, reverseRead ? 'true' : 'false');
    document.getElementById('setRtl').classList.toggle('on', reverseRead);
}}

function toggleClick() {{
    enableClick = !enableClick;
    localStorage.setItem(KEY_CLICK, enableClick ? 'true' : 'false');
    document.getElementById('setClick').classList.toggle('on', enableClick);
}}

function toggleTouch() {{
    enableTouch = !enableTouch;
    localStorage.setItem(KEY_TOUCH, enableTouch ? 'true' : 'false');
    document.getElementById('setTouch').classList.toggle('on', enableTouch);
}}

document.addEventListener('DOMContentLoaded', function() {{
    const viewer = document.getElementById('viewer');
    if (viewer) viewer.addEventListener('scroll', handleScrollEvent);
}});

(function init() {{
    const savedMode = localStorage.getItem(KEY_MODE);
    if (savedMode === 'scroll') flipMode = false;
    current = loadProgress();
    applyMode();
    if (flipMode) {{
        showPage(current);
    }} else {{
        setTimeout(() => {{
            const v = document.getElementById('viewer');
            if (v) v.addEventListener('scroll', handleScrollEvent);
        }}, 200);
    }}
}})();

showBars();
</script>
</body>
</html>
"#,
        name = name,
        total = total,
        images_json = images_json,
        name_json = name_json,
        album_path_json = album_path_json,
    )
}