use crate::scanner::{BrowseResult, Config};
use chrono::DateTime;
use std::collections::HashMap;
use super::shared::{url_encode, html_escape, DEFAULT_COVER};

pub fn render_index(browse: &BrowseResult, config: &Config, views: &HashMap<String, u64>) -> String {
    let accent_color = &config.primary_color;

    let folder_cards: String = browse.folders.iter().map(|folder| {
        let url = format!("/?path={}", url_encode(&folder.path));
        format!(r#"<a class="folder-card" href="{url}" data-name="{n}"><div class="folder-icon">📁</div><div class="folder-info"><div class="folder-title">{name}</div><div class="folder-count">{c} 个子项</div></div></a>"#,
            url = url, n = html_escape(&folder.name.to_lowercase()),
            name = html_escape(&folder.name), c = folder.child_count)
    }).collect();

    let album_cards: String = browse.albums.iter().map(|album| {
        let cover_url = album.cover.as_ref().map(|c| format!("/raw?path={}", url_encode(c))).unwrap_or_else(|| DEFAULT_COVER.to_string());
        let viewer_url = format!("/viewer?path={}", url_encode(&album.path));
        let time_str = if album.modified > 0 {
            DateTime::from_timestamp(album.modified as i64, 0).map(|dt| dt.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "未知".to_string())
        } else { "未知".to_string() };
        let v = views.get(&album.path).copied().unwrap_or(0);
        let views_text = if v > 0 { format!("{} 次", v) } else { String::new() };
        format!(r#"<a class="album-card" href="{v}" data-name="{nl}" data-path="{path}" data-modified="{m}" data-cover="{cover}" data-count="{c}" data-views="{views}"><div class="cover"><img src="{cover}" alt="{name}" loading="lazy" decoding="async"><div class="progress-bar" style="display:none"><div class="progress-fill"></div></div></div><div class="info"><div class="title" title="{name}">{name}</div><div class="meta"><span class="count">{c} 页</span><span class="views-text">{vt}</span><span class="progress-text"></span></div><div class="time">{t}</div></div></a>"#,
            v = viewer_url, cover = cover_url, name = html_escape(&album.name),
            nl = html_escape(&album.name.to_lowercase()), path = html_escape(&album.path),
            c = album.image_count, m = album.modified, t = html_escape(&time_str),
            views = v, vt = views_text)
    }).collect();

    let breadcrumbs_html = if browse.breadcrumbs.len() > 1 {
        let items: Vec<String> = browse.breadcrumbs.iter().enumerate().map(|(i, bc)| {
            if i == browse.breadcrumbs.len() - 1 {
                format!(r#"<span class="bc-current">{}</span>"#, html_escape(&bc.name))
            } else {
                format!(r#"<a href="/?path={}">{}</a>"#, url_encode(&bc.path), html_escape(&bc.name))
            }
        }).collect();
        format!(r#"<div class="breadcrumbs">{}</div>"#, items.join(r#"<span class="bc-sep">/</span>"#))
    } else { String::new() };

    let has_content = !browse.folders.is_empty() || !browse.albums.is_empty();
    let empty_hint = if !has_content { r#"<div class="empty">📭 当前目录下没有找到漫画或文件夹</div>"# } else { "" };
    let folder_section = if !browse.folders.is_empty() {
        format!(r#"<div class="folder-section"><div class="section-label">文件夹</div><div class="folder-grid" id="folderGrid">{}</div></div>"#, folder_cards)
    } else { String::new() };

    let total = browse.albums.len();

    let popup_css = if config.popup_enabled { r#"
.popup-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); backdrop-filter: blur(6px); display: none; align-items: center; justify-content: center; z-index: 9999; }
.popup-overlay.show { display: flex; animation: fadeIn 0.25s ease; }
.popup-container { background: var(--container-bg); border-radius: 14px; max-width: 460px; width: 90%; box-shadow: 0 20px 60px rgba(0,0,0,0.3); overflow: hidden; border: 1px solid var(--border-color); animation: slideUp 0.3s ease; }
.popup-header { padding: 18px 22px 14px; border-bottom: 1px solid var(--border-color); display: flex; justify-content: space-between; align-items: center; }
.popup-header h2 { font-size: 1.15rem; color: var(--accent); margin: 0; font-weight: 600; }
.popup-close { background: none; border: none; font-size: 24px; color: var(--text-secondary); cursor: pointer; line-height: 1; }
.popup-body { padding: 20px 22px; color: var(--text-primary); line-height: 1.7; }
.popup-body p { margin: 0; white-space: pre-wrap; word-break: break-word; }
.popup-footer { padding: 14px 22px 18px; border-top: 1px solid var(--border-color); display: flex; justify-content: flex-end; gap: 10px; }
.popup-btn { padding: 8px 20px; border: none; border-radius: 8px; font-size: 0.9rem; cursor: pointer; }
.popup-btn.confirm { background: var(--accent); color: #fff; }
.popup-btn.cancel { background: var(--card-bg); color: var(--text-secondary); border: 1px solid var(--border-color); }
"# } else { "" };

    let popup_html = if config.popup_enabled {
        format!(r#"<div id="popupOverlay" class="popup-overlay"><div class="popup-container"><div class="popup-header"><h2>{t}</h2><button class="popup-close" onclick="closePopup()">&times;</button></div><div class="popup-body"><p>{c}</p></div><div class="popup-footer">{cb}<button class="popup-btn confirm" onclick="closePopup()">{cf}</button></div></div></div>"#,
            t = html_escape(&config.popup_title), c = html_escape(&config.popup_content).replace("\n", "<br>"),
            cf = html_escape(&config.popup_confirm_text),
            cb = if !config.popup_cancel_text.is_empty() {
                format!(r#"<button class="popup-btn cancel" onclick="closePopup()">{}</button>"#, html_escape(&config.popup_cancel_text))
            } else { "".to_string() })
    } else { "".to_string() };

    let popup_js = if config.popup_enabled {
        let so = if config.popup_show_once { "true" } else { "false" };
        format!(r#"
const POPUP_KEY = 'nwebp_popup_shown';
const POPUP_SHOW_ONCE = {so};
function showPopup() {{ const o = document.getElementById('popupOverlay'); if (!o) return;
    if (POPUP_SHOW_ONCE && localStorage.getItem(POPUP_KEY) === 'true') return;
    o.classList.add('show'); document.body.style.overflow = 'hidden'; }}
function closePopup() {{ const o = document.getElementById('popupOverlay'); if (!o) return;
    o.classList.remove('show'); document.body.style.overflow = '';
    if (POPUP_SHOW_ONCE) localStorage.setItem(POPUP_KEY, 'true'); }}
document.addEventListener('click', function(e) {{ const o = document.getElementById('popupOverlay'); if (o && e.target === o) closePopup(); }});
document.addEventListener('keydown', function(e) {{ if (e.key === 'Escape') closePopup(); }});
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', showPopup); else showPopup();
"#, so = so)
    } else { "".to_string() };

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
    --text-primary: #2c2c2c;
    --text-secondary: #6b7280;
    --card-bg: #ffffff;
    --card-shadow: rgba(0,0,0,0.08);
    --accent: {accent};
    --search-bg: #f3f4f6;
    --cover-bg: #e5e7eb;
    --stats-bg: #f9fafb;
    --border-color: #e5e7eb;
    --folder-bg: #eef2ff;
    --hover-bg: #f3f4f6;
    --bg-image: url("https://www.loliapi.com/acg/");
    --container-alpha: 0.4;
    --bg-blur: 5px;
    --btn-bg: rgba(255,255,255,0.65);
    --btn-border: rgba(255,255,255,0.5);
}}
[data-theme="night"] {{
    --bg: #1a1a2e;
    --container-bg: #242438;
    --text-primary: #e8e8f0;
    --text-secondary: #9ca3af;
    --card-bg: #2e2e48;
    --card-shadow: rgba(0,0,0,0.3);
    --accent: #7ab0e0;
    --search-bg: #2e2e48;
    --cover-bg: #1f1f33;
    --stats-bg: #2a2a42;
    --border-color: #3a3a52;
    --folder-bg: #2a3050;
    --hover-bg: #3a3a52;
    --btn-bg: rgba(46,46,72,0.7);
    --btn-border: rgba(255,255,255,0.15);
}}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
html {{ scroll-behavior: smooth; }}
body {{
    min-height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
    background: var(--bg);
    color: var(--text-primary);
    font-size: 14px;
    transition: background-color 0.3s ease, color 0.3s ease;
    position: relative;
}}
body[data-bg="on"] {{ background-color: transparent; }}
body[data-bg="on"]::before {{
    content: '';
    position: fixed;
    inset: -40px;
    background: var(--bg-image) center/cover no-repeat;
    filter: blur(var(--bg-blur)) brightness(0.95) saturate(1.15);
    z-index: -2;
    pointer-events: none;
    transition: filter 0.3s ease;
}}
body[data-theme="night"][data-bg="on"]::before {{
    filter: blur(var(--bg-blur)) brightness(0.65) saturate(0.95);
}}
body[data-bg="on"]::after {{
    content: '';
    position: fixed;
    inset: 0;
    background: rgba(255,255,255,0.05);
    z-index: -1;
    pointer-events: none;
}}
body[data-theme="night"][data-bg="on"]::after {{ background: rgba(0,0,0,0.15); }}
.container {{
    max-width: 1400px; margin: 20px auto; padding: 24px;
    background: var(--container-bg); border-radius: 14px;
    box-shadow: 0 2px 16px rgba(0,0,0,0.08);
    transition: background-color 0.3s ease;
}}
body[data-bg="on"] .container {{
    background: rgba(255,255,255, var(--container-alpha));
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
}}
body[data-theme="night"][data-bg="on"] .container {{
    background: rgba(30,30,48, var(--container-alpha));
}}
.header {{
    display: flex; justify-content: space-between; align-items: center;
    flex-wrap: wrap; gap: 12px; margin-bottom: 16px; padding-bottom: 14px;
    border-bottom: 1px solid var(--border-color);
}}
.header-left h1 {{ font-size: 22px; font-weight: 700; color: var(--accent); }}
.header-left .subtitle {{ font-size: 12px; color: var(--text-secondary); margin-top: 2px; }}
.header-right {{ display: flex; gap: 8px; align-items: center; flex-wrap: wrap; position: relative; }}
.icon-btn {{
    background: var(--btn-bg);
    border: 1px solid var(--btn-border);
    border-radius: 8px; padding: 7px 12px; cursor: pointer;
    font-size: 13px; color: var(--text-primary); white-space: nowrap;
    transition: all 0.2s ease; display: inline-flex; align-items: center; gap: 4px;
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
}}
.icon-btn:hover {{ background: var(--hover-bg); border-color: var(--accent); transform: translateY(-1px); }}
body[data-bg="on"] .icon-btn:hover {{ background: rgba(255,255,255,0.85); }}
body[data-theme="night"][data-bg="on"] .icon-btn:hover {{ background: rgba(58,58,82,0.9); }}
.search-box {{
    display: flex; align-items: center; gap: 6px;
    background: var(--btn-bg);
    border: 1px solid var(--btn-border);
    border-radius: 8px; padding: 5px 12px;
    transition: border-color 0.2s ease;
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
}}
.search-box:focus-within {{ border-color: var(--accent); }}
.search-box input {{
    border: none; outline: none; background: transparent;
    font-size: 13px; width: 170px; color: var(--text-primary);
}}
.breadcrumbs {{
    margin-bottom: 14px; font-size: 13px;
    display: flex; flex-wrap: wrap; align-items: center; gap: 4px;
    color: var(--text-secondary);
}}
.breadcrumbs a {{
    color: var(--accent); text-decoration: none;
    padding: 2px 8px; border-radius: 6px;
}}
.breadcrumbs a:hover {{ background: var(--hover-bg); }}
.bc-current {{ color: var(--text-primary); font-weight: 600; padding: 2px 4px; }}
.bc-sep {{ color: var(--text-secondary); opacity: 0.5; }}
.toolbar {{
    display: flex; gap: 10px; align-items: center; flex-wrap: wrap;
    margin-bottom: 16px; padding: 10px 12px;
    background: var(--stats-bg); border-radius: 10px;
    border: 1px solid var(--border-color);
    transition: background-color 0.3s ease;
}}
body[data-bg="on"] .toolbar {{
    background: var(--btn-bg);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
}}
.toolbar-group {{ display: flex; align-items: center; gap: 6px; }}
.toolbar-label {{ font-size: 12px; color: var(--text-secondary); }}
.toolbar select {{
    padding: 5px 10px; border-radius: 6px;
    border: 1px solid var(--border-color);
    background: var(--btn-bg);
    color: var(--text-primary); font-size: 13px; cursor: pointer; outline: none;
    backdrop-filter: blur(4px);
}}
body[data-bg="on"] .toolbar select {{ background: rgba(255,255,255,0.7); }}
body[data-theme="night"][data-bg="on"] .toolbar select {{ background: rgba(58,58,82,0.75); }}
.seg-btn {{
    background: var(--btn-bg);
    border: 1px solid var(--btn-border);
    border-radius: 6px; padding: 5px 10px; cursor: pointer;
    color: var(--text-secondary); font-size: 13px;
    transition: all 0.2s ease;
    backdrop-filter: blur(4px);
}}
.seg-btn:hover {{ color: var(--text-primary); }}
.seg-btn.active {{ background: var(--accent); color: #fff; border-color: var(--accent); }}
.section-label {{
    font-size: 12px; color: var(--text-secondary);
    margin-bottom: 8px; padding-left: 4px;
    text-transform: uppercase; letter-spacing: 0.5px;
}}
.folder-section {{ margin-bottom: 20px; }}
.folder-grid {{
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 10px;
}}
.folder-card {{
    text-decoration: none; color: inherit;
    background: var(--folder-bg); border-radius: 10px;
    border: 1px solid var(--border-color); padding: 12px 14px;
    display: flex; align-items: center; gap: 10px;
    transition: all 0.2s ease; cursor: pointer;
}}
body[data-bg="on"] .folder-card {{
    background: rgba(238,242,255, calc(var(--container-alpha) + 0.25));
    backdrop-filter: blur(4px);
}}
body[data-theme="night"][data-bg="on"] .folder-card {{
    background: rgba(42,48,80, calc(var(--container-alpha) + 0.25));
}}
.folder-card:hover {{
    transform: translateY(-2px);
    box-shadow: 0 4px 12px var(--card-shadow);
    border-color: var(--accent);
}}
.folder-icon {{ font-size: 26px; flex-shrink: 0; }}
.folder-info {{ flex: 1; min-width: 0; }}
.folder-title {{ font-size: 13px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-bottom: 2px; }}
.folder-count {{ font-size: 11px; color: var(--accent); }}
.album-grid {{
    display: grid; gap: 14px; justify-items: stretch;
    transition: grid-template-columns 0.25s ease;
}}
.album-card {{
    text-decoration: none; color: inherit;
    background: var(--card-bg); border-radius: 10px;
    overflow: hidden; border: 1px solid var(--border-color);
    transition: all 0.2s ease; cursor: pointer;
    display: flex; flex-direction: column;
    box-shadow: 0 1px 3px var(--card-shadow);
}}
body[data-bg="on"] .album-card {{
    background: rgba(255,255,255, calc(var(--container-alpha) + 0.35));
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
}}
body[data-theme="night"][data-bg="on"] .album-card {{
    background: rgba(46,46,72, calc(var(--container-alpha) + 0.35));
}}
.album-card:hover {{
    transform: translateY(-3px);
    box-shadow: 0 8px 20px var(--card-shadow);
    border-color: var(--accent);
}}
.album-card.hidden {{ display: none; }}
.album-card .cover {{ width: 100%; aspect-ratio: 2/3; overflow: hidden; background: var(--cover-bg); position: relative; }}
.album-card .cover img {{ width: 100%; height: 100%; object-fit: cover; display: block; transition: transform 0.3s ease; }}
.album-card:hover .cover img {{ transform: scale(1.03); }}
.progress-bar {{ position: absolute; bottom: 0; left: 0; right: 0; height: 4px; background: rgba(0,0,0,0.4); }}
.progress-fill {{ height: 100%; background: var(--accent); width: 0%; transition: width 0.3s ease; }}
.album-card .info {{ padding: 10px 11px; display: flex; flex-direction: column; gap: 3px; }}
.album-card .title {{ font-size: 13px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; line-height: 1.3; }}
.album-card .meta {{ display: flex; justify-content: space-between; align-items: center; font-size: 11px; gap: 6px; }}
.album-card .count {{ color: var(--accent); }}
.album-card .views-text {{ color: #f59e0b; font-size: 10px; }}
.album-card .progress-text {{ color: var(--text-secondary); font-size: 10px; }}
.album-card .time {{ font-size: 10px; color: var(--text-secondary); }}
body[data-view="list"] .album-grid {{ grid-template-columns: 1fr !important; }}
body[data-view="list"] .album-card {{ flex-direction: row; align-items: center; padding: 10px; gap: 14px; }}
body[data-view="list"] .album-card .cover {{ width: 70px; height: 100px; aspect-ratio: auto; border-radius: 6px; flex-shrink: 0; }}
body[data-view="list"] .album-card .info {{ flex: 1; padding: 0; }}
body[data-view="list"] .album-card .time {{ display: none; }}
.empty, .no-result {{ text-align: center; padding: 50px 20px; color: var(--text-secondary); font-size: 15px; }}
.footer {{ margin-top: 24px; padding-top: 16px; border-top: 1px solid var(--border-color); text-align: center; font-size: 12px; color: var(--text-secondary); }}

.history-panel, .stats-panel {{
    position: absolute; top: 100%; right: 0; margin-top: 8px;
    background: var(--container-bg); border: 1px solid var(--border-color);
    border-radius: 10px; box-shadow: 0 12px 32px rgba(0,0,0,0.15);
    min-width: 260px; max-width: 400px; max-height: 400px;
    overflow-y: auto; z-index: 100; padding: 8px;
    display: none;
}}
body[data-bg="on"] .history-panel, body[data-bg="on"] .stats-panel {{
    background: rgba(255,255,255, 0.96);
    backdrop-filter: blur(8px);
}}
body[data-theme="night"][data-bg="on"] .history-panel,
body[data-theme="night"][data-bg="on"] .stats-panel {{
    background: rgba(36,36,56, 0.96);
}}
.history-panel.show, .stats-panel.show {{ display: block; animation: fadeIn 0.2s ease; }}
.history-header, .stats-header {{
    font-size: 11px; color: var(--text-secondary); padding: 8px 10px;
    font-weight: 600; border-bottom: 1px solid var(--border-color);
    margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.5px;
}}
.history-item {{
    display: block; padding: 8px 10px; border-radius: 6px;
    color: var(--text-primary); text-decoration: none; font-size: 13px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}}
.history-item:hover {{ background: var(--hover-bg); color: var(--accent); }}
.history-empty {{ padding: 20px; text-align: center; color: var(--text-secondary); font-size: 13px; }}

.stats-panel {{ min-width: 300px; }}
.stats-grid {{
    display: grid; grid-template-columns: repeat(3, 1fr);
    gap: 8px; padding: 10px 6px;
}}
.stat-item {{
    text-align: center; padding: 10px 6px;
    background: var(--stats-bg); border-radius: 8px;
    border: 1px solid var(--border-color);
}}
.stat-label {{ font-size: 11px; color: var(--text-secondary); margin-bottom: 4px; }}
.stat-value {{ font-size: 15px; font-weight: 700; color: var(--accent); word-break: break-all; }}

.settings-overlay {{ position: fixed; inset: 0; background: rgba(0,0,0,0.5); backdrop-filter: blur(6px); display: none; align-items: center; justify-content: center; z-index: 9998; }}
.settings-overlay.show {{ display: flex; animation: fadeIn 0.25s ease; }}
.settings-modal {{ background: var(--container-bg); border-radius: 14px; max-width: 500px; width: 90%; box-shadow: 0 20px 60px rgba(0,0,0,0.3); overflow: hidden; border: 1px solid var(--border-color); animation: slideUp 0.3s ease; max-height: 88vh; display: flex; flex-direction: column; }}
body[data-bg="on"] .settings-modal {{ background: rgba(255,255,255,0.98); }}
body[data-theme="night"][data-bg="on"] .settings-modal {{ background: rgba(36,36,56,0.98); }}
.settings-header {{ padding: 18px 22px 14px; border-bottom: 1px solid var(--border-color); display: flex; justify-content: space-between; align-items: center; }}
.settings-header h2 {{ font-size: 1.1rem; color: var(--accent); margin: 0; font-weight: 600; }}
.settings-close {{ background: none; border: none; font-size: 24px; color: var(--text-secondary); cursor: pointer; line-height: 1; }}
.settings-body {{ padding: 16px 22px; display: flex; flex-direction: column; gap: 14px; overflow-y: auto; }}
.setting-row {{ display: flex; justify-content: space-between; align-items: center; gap: 12px; }}
.setting-label {{ font-size: 13px; color: var(--text-primary); flex: 1; }}
.setting-label small {{ display: block; color: var(--text-secondary); font-size: 11px; margin-top: 2px; }}
.setting-control {{ display: flex; align-items: center; gap: 8px; }}
.setting-control select,
.setting-control input[type="text"] {{
    padding: 5px 10px; border-radius: 6px;
    border: 1px solid var(--border-color); background: var(--card-bg);
    color: var(--text-primary); font-size: 13px; outline: none; min-width: 90px;
}}
.setting-control input[type="range"] {{
    -webkit-appearance: none; appearance: none;
    width: 130px; height: 4px;
    background: var(--border-color); border-radius: 2px;
    outline: none; cursor: pointer;
}}
.setting-control input[type="range"]::-webkit-slider-thumb {{
    -webkit-appearance: none; appearance: none;
    width: 16px; height: 16px;
    background: var(--accent); border-radius: 50%; cursor: pointer;
}}
.setting-control input[type="range"]::-moz-range-thumb {{
    width: 16px; height: 16px;
    background: var(--accent); border-radius: 50%; cursor: pointer; border: none;
}}
.setting-control .range-val {{
    font-size: 12px; color: var(--text-secondary);
    min-width: 42px; text-align: right;
}}
.toggle-switch {{ position: relative; width: 40px; height: 22px; background: var(--border-color); border-radius: 11px; cursor: pointer; transition: background-color 0.25s ease; flex-shrink: 0; }}
.toggle-switch.on {{ background: var(--accent); }}
.toggle-switch::after {{ content: ''; position: absolute; top: 2px; left: 2px; width: 18px; height: 18px; background: #fff; border-radius: 50%; transition: transform 0.25s ease; }}
.toggle-switch.on::after {{ transform: translateX(18px); }}
.settings-section-title {{ font-size: 12px; color: var(--text-secondary); font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; padding: 8px 0 4px 0; border-top: 1px solid var(--border-color); margin-top: 4px; }}
.clear-options {{ display: grid; grid-template-columns: 1fr 1fr; gap: 8px 14px; padding: 8px 0; }}
.clear-options label {{ display: flex; align-items: center; gap: 6px; font-size: 13px; cursor: pointer; color: var(--text-primary); }}
.clear-options input[type="checkbox"] {{ width: 16px; height: 16px; accent-color: var(--accent); cursor: pointer; }}
.clear-actions {{ display: flex; gap: 8px; padding-top: 6px; flex-wrap: wrap; }}
.settings-footer {{ padding: 12px 22px 18px; display: flex; justify-content: space-between; gap: 10px; border-top: 1px solid var(--border-color); }}
.danger-btn {{ padding: 8px 16px; border-radius: 8px; background: #ef4444; color: #fff; border: none; cursor: pointer; font-size: 13px; }}
.danger-btn:hover {{ background: #dc2626; }}
.danger-btn.soft {{ background: transparent; color: #ef4444; border: 1px solid #ef4444; }}
.danger-btn.soft:hover {{ background: #ef4444; color: #fff; }}
.primary-btn {{ padding: 8px 20px; border-radius: 8px; background: var(--accent); color: #fff; border: none; cursor: pointer; font-size: 13px; }}
.primary-btn.small {{ padding: 5px 12px; font-size: 12px; }}
.back-to-top {{
    position: fixed; bottom: 30px; right: 30px;
    width: 44px; height: 44px; border-radius: 50%;
    background: var(--accent); color: #fff; border: none;
    font-size: 20px; cursor: pointer; z-index: 200;
    box-shadow: 0 4px 16px rgba(0,0,0,0.25);
    opacity: 0; visibility: hidden; transform: translateY(10px);
    transition: all 0.3s ease;
}}
.back-to-top.show {{ opacity: 1; visibility: visible; transform: translateY(0); }}
.back-to-top:hover {{ transform: translateY(-3px); }}
@keyframes fadeIn {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
@keyframes slideUp {{ from {{ opacity: 0; transform: translateY(30px) scale(0.96); }} to {{ opacity: 1; transform: translateY(0) scale(1); }} }}
{popup_css}
@media (max-width: 768px) {{
    .container {{ margin: 10px; padding: 16px; }}
    .header-left h1 {{ font-size: 18px; }}
    .search-box input {{ width: 90px; }}
    .folder-grid {{ grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); }}
    .settings-modal {{ max-width: 95%; }}
    .clear-options {{ grid-template-columns: 1fr; }}
    .back-to-top {{ bottom: 20px; right: 20px; width: 40px; height: 40px; }}
    .stats-grid {{ grid-template-columns: 1fr; }}
    .setting-control input[type="range"] {{ width: 90px; }}
}}
</style>
</head>
<body data-view="grid" data-bg="off">
<div class="container">
    <div class="header">
        <div class="header-left">
            <h1>{title}</h1>
            <div class="subtitle">{subtitle}</div>
        </div>
        <div class="header-right">
            <div class="search-box">
                <span>🔍</span>
                <input type="text" id="searchInput" placeholder="搜索全部..." oninput="filterAlbums()">
            </div>
            <button class="icon-btn" id="historyBtn" onclick="toggleHistory(event)">📜 历史</button>
            <button class="icon-btn" id="statsBtn" onclick="toggleStats(event)">📊 统计</button>
            <button class="icon-btn" onclick="randomAlbum()">🎲 随机</button>
            <button class="icon-btn" onclick="refreshPage()" title="刷新">🔄 刷新</button>
            <button class="icon-btn" id="themeBtn" onclick="toggleTheme()">🌙 夜间</button>
            <button class="icon-btn" onclick="openSettings()">⚙️ 设置</button>
            <div class="history-panel" id="historyPanel">
                <div class="history-header">最近阅读</div>
                <div id="historyList"></div>
            </div>
            <div class="stats-panel" id="statsPanel">
                <div class="stats-header">库统计</div>
                <div class="stats-grid">
                    <div class="stat-item">
                        <div class="stat-label">漫画</div>
                        <div class="stat-value" id="statAlbums">-</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">图片</div>
                        <div class="stat-value" id="statImages">-</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">大小</div>
                        <div class="stat-value" id="statSize">-</div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    {breadcrumbs_html}
    <div class="toolbar">
        <div class="toolbar-group">
            <span class="toolbar-label">排序</span>
            <select id="sortSelect" onchange="sortAlbums(this.value)">
                <option value="name">名称 ↑</option>
                <option value="name-desc">名称 ↓</option>
                <option value="time">时间 ↑</option>
                <option value="time-desc">时间 ↓</option>
                <option value="count">数量 ↑</option>
                <option value="count-desc">数量 ↓</option>
                <option value="views">浏览 ↑</option>
                <option value="views-desc">浏览 ↓</option>
            </select>
        </div>
        <div class="toolbar-group">
            <span class="toolbar-label">列数</span>
            <select id="colsSelect" onchange="changeColumns(this.value)">
                <option value="2">2 列</option>
                <option value="3">3 列</option>
                <option value="4">4 列</option>
                <option value="5">5 列</option>
                <option value="6">6 列</option>
            </select>
        </div>
        <div class="toolbar-group">
            <button class="seg-btn" id="viewGrid" onclick="setView('grid')">网格</button>
            <button class="seg-btn" id="viewList" onclick="setView('list')">列表</button>
        </div>
        <div class="toolbar-group">
            <span class="toolbar-label">显示预览图</span>
            <div class="toggle-switch" id="coverToggle" onclick="toggleCover()"></div>
        </div>
        <div class="toolbar-group" style="margin-left:auto">
            <span class="toolbar-label">共 {total} 本 · 显示 <span id="visibleCount">{total}</span></span>
        </div>
    </div>
    {folder_section}
    {empty_hint}
    <div class="album-grid" id="albumGrid">{cards}</div>
    <div class="no-result" id="noResult" style="display:none">😢 没有找到匹配的漫画</div>
    <div class="footer"><p>{footer}</p></div>
</div>

<button class="back-to-top" id="backTop" onclick="scrollToTop()" title="返回顶部">↑</button>

{popup_html}

<div class="settings-overlay" id="settingsOverlay" onclick="if(event.target===this)closeSettings()">
    <div class="settings-modal">
        <div class="settings-header">
            <h2>⚙️ 设置</h2>
            <button class="settings-close" onclick="closeSettings()">&times;</button>
        </div>
        <div class="settings-body">
            <div class="setting-row">
                <div class="setting-label">夜间模式<small>切换明暗主题</small></div>
                <div class="toggle-switch" id="setTheme" onclick="toggleTheme()"></div>
            </div>
            <div class="setting-row">
                <div class="setting-label">显示预览图<small>加载漫画封面</small></div>
                <div class="toggle-switch" id="setCover" onclick="toggleCover()"></div>
            </div>
            <div class="setting-row">
                <div class="setting-label">网格列数<small>主页卡片每行数量</small></div>
                <div class="setting-control">
                    <select id="setCols" onchange="changeColumns(this.value)">
                        <option value="2">2 列</option>
                        <option value="3">3 列</option>
                        <option value="4">4 列</option>
                        <option value="5">5 列</option>
                        <option value="6">6 列</option>
                    </select>
                </div>
            </div>
            <div class="setting-row">
                <div class="setting-label">视图模式<small>网格或列表</small></div>
                <div class="setting-control">
                    <select id="setView" onchange="setView(this.value)">
                        <option value="grid">网格</option>
                        <option value="list">列表</option>
                    </select>
                </div>
            </div>
            <div class="setting-row">
                <div class="setting-label">排序方式<small>漫画排列顺序</small></div>
                <div class="setting-control">
                    <select id="setSort" onchange="sortAlbums(this.value)">
                        <option value="name">名称 ↑</option>
                        <option value="name-desc">名称 ↓</option>
                        <option value="time">时间 ↑</option>
                        <option value="time-desc">时间 ↓</option>
                        <option value="count">数量 ↑</option>
                        <option value="count-desc">数量 ↓</option>
                        <option value="views">浏览 ↑</option>
                        <option value="views-desc">浏览 ↓</option>
                    </select>
                </div>
            </div>

            <div class="settings-section-title">背景图</div>
            <div class="setting-row">
                <div class="setting-label">开启背景图<small>模糊效果，默认 loliapi.com/acg</small></div>
                <div class="toggle-switch" id="setBgEnabled" onclick="toggleBg()"></div>
            </div>
            <div class="setting-row">
                <div class="setting-label">背景图链接<small>留空使用默认</small></div>
                <div class="setting-control">
                    <input type="text" id="bgUrlInput" style="width:130px" placeholder="图片URL">
                </div>
            </div>
            <div style="display:flex;gap:8px;flex-wrap:wrap">
                <button class="primary-btn small" onclick="applyBgUrl()">应用链接</button>
                <button class="primary-btn small" onclick="resetBgUrl()">重置默认</button>
            </div>
            <div class="setting-row">
                <div class="setting-label">背景模糊度<small>0 清晰 · 40 最模糊</small></div>
                <div class="setting-control">
                    <input type="range" id="setBlur" min="0" max="40" value="5" oninput="applyBlur(this.value)" onchange="saveBlur(this.value)">
                    <span class="range-val" id="blurVal">5px</span>
                </div>
            </div>
            <div class="setting-row">
                <div class="setting-label">容器透明度<small>10% 通透 · 100% 不透明</small></div>
                <div class="setting-control">
                    <input type="range" id="setAlpha" min="10" max="100" value="40" oninput="applyAlpha(this.value)" onchange="saveAlpha(this.value)">
                    <span class="range-val" id="alphaVal">40%</span>
                </div>
            </div>

            <div class="settings-section-title">清除缓存</div>
            <div class="clear-options" id="clearOptions">
                <label><input type="checkbox" data-clear="theme"> 主题</label>
                <label><input type="checkbox" data-clear="view"> 视图</label>
                <label><input type="checkbox" data-clear="columns"> 列数</label>
                <label><input type="checkbox" data-clear="sort"> 排序</label>
                <label><input type="checkbox" data-clear="cover"> 预览图</label>
                <label><input type="checkbox" data-clear="history"> 浏览历史</label>
                <label><input type="checkbox" data-clear="progress"> 阅读进度</label>
                <label><input type="checkbox" data-clear="viewer"> 阅读设置</label>
                <label><input type="checkbox" data-clear="bg"> 背景图</label>
                <label><input type="checkbox" data-clear="popup"> 公告已读</label>
            </div>
            <div class="clear-actions">
                <button class="danger-btn soft" onclick="selectAllClear()">全选</button>
                <button class="danger-btn soft" onclick="unselectAllClear()">取消全选</button>
                <button class="danger-btn soft" onclick="clearSelected()">清除选中项</button>
            </div>
        </div>
        <div class="settings-footer">
            <button class="danger-btn" onclick="clearAll()">🗑️ 清除全部缓存</button>
            <button class="primary-btn" onclick="closeSettings()">完成</button>
        </div>
    </div>
</div>

<script>
const DEFAULT_COVER_JS = 'data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 width=%22200%22 height=%22300%22%3E%3Crect fill=%22%23e0e0e0%22 width=%22200%22 height=%22300%22/%3E%3Ctext fill=%22%23999%22 x=%22100%22 y=%22150%22 text-anchor=%22middle%22%3E无封面%3C/text%3E%3C/svg%3E';

const KEY_SORT = 'nwebp_sort';
const KEY_VIEW = 'nwebp_view';
const KEY_COVER = 'nwebp_show_cover';
const KEY_COLS = 'nwebp_columns';
const KEY_THEME = 'nwebp_theme';
const KEY_HISTORY = 'nwebp_history';
const KEY_BG_ENABLED = 'nwebp_bg_enabled';
const KEY_BG_URL = 'nwebp_bg_url';
const KEY_BG_BLUR = 'nwebp_bg_blur';
const KEY_BG_ALPHA = 'nwebp_container_alpha';
const DEFAULT_BG_URL = 'https://www.loliapi.com/acg/';
const DEFAULT_BLUR = 5;
const DEFAULT_ALPHA = 40;

let allAlbumsCache = [];
let showCover = localStorage.getItem(KEY_COVER) === 'true';
let showSearchResults = false;

function escapeHtml(s) {{
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}}

function formatTime(ts) {{
    if (!ts) return '未知';
    const d = new Date(ts * 1000);
    const pad = n => String(n).padStart(2, '0');
    return d.getFullYear() + '-' + pad(d.getMonth()+1) + '-' + pad(d.getDate()) + ' ' + pad(d.getHours()) + ':' + pad(d.getMinutes());
}}

function formatSize(bytes) {{
    if (!bytes) return '0 B';
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    if (bytes < 1024 * 1024 * 1024) return (bytes / 1024 / 1024).toFixed(1) + ' MB';
    return (bytes / 1024 / 1024 / 1024).toFixed(2) + ' GB';
}}

function albumCardHtml(album) {{
    const cover = album.cover ? '/raw?path=' + encodeURIComponent(album.cover) : DEFAULT_COVER_JS;
    const displayCover = showCover ? cover : DEFAULT_COVER_JS;
    const nameLower = album.name.toLowerCase();
    const viewerUrl = '/viewer?path=' + encodeURIComponent(album.path);
    const views = album.views || 0;
    const viewsText = views > 0 ? (views + ' 次') : '';
    return '<a class="album-card" href="' + viewerUrl + '"' +
        ' data-name="' + escapeHtml(nameLower) + '"' +
        ' data-path="' + escapeHtml(album.path) + '"' +
        ' data-modified="' + album.modified + '"' +
        ' data-cover="' + escapeHtml(cover) + '"' +
        ' data-count="' + album.image_count + '"' +
        ' data-views="' + views + '">' +
        '<div class="cover">' +
        '<img src="' + displayCover + '" alt="' + escapeHtml(album.name) + '" loading="lazy" decoding="async">' +
        '<div class="progress-bar" style="display:none"><div class="progress-fill"></div></div>' +
        '</div>' +
        '<div class="info">' +
        '<div class="title" title="' + escapeHtml(album.name) + '">' + escapeHtml(album.name) + '</div>' +
        '<div class="meta"><span class="count">' + album.image_count + ' 页</span><span class="views-text">' + viewsText + '</span><span class="progress-text"></span></div>' +
        '<div class="time">' + formatTime(album.modified) + '</div>' +
        '</div>' +
        '</a>';
}}

/* ============ 背景图 ============ */
function applyBg() {{
    const enabled = localStorage.getItem(KEY_BG_ENABLED) === 'true';
    const url = localStorage.getItem(KEY_BG_URL) || DEFAULT_BG_URL;
    const blur = parseInt(localStorage.getItem(KEY_BG_BLUR));
    const blurVal = isNaN(blur) ? DEFAULT_BLUR : blur;
    const alpha = parseInt(localStorage.getItem(KEY_BG_ALPHA));
    const alphaVal = isNaN(alpha) ? DEFAULT_ALPHA : alpha;

    document.body.style.setProperty('--bg-image', 'url(' + JSON.stringify(url) + ')');
    document.documentElement.style.setProperty('--bg-blur', blurVal + 'px');
    document.documentElement.style.setProperty('--container-alpha', (alphaVal / 100).toString());
    document.body.dataset.bg = enabled ? 'on' : 'off';

    const el = document.getElementById('setBgEnabled');
    if (el) el.classList.toggle('on', enabled);
    const input = document.getElementById('bgUrlInput');
    if (input && document.activeElement !== input) input.value = url;
    const blurInput = document.getElementById('setBlur');
    if (blurInput && document.activeElement !== blurInput) blurInput.value = blurVal;
    const blurValEl = document.getElementById('blurVal');
    if (blurValEl) blurValEl.textContent = blurVal + 'px';
    const alphaInput = document.getElementById('setAlpha');
    if (alphaInput && document.activeElement !== alphaInput) alphaInput.value = alphaVal;
    const alphaValEl = document.getElementById('alphaVal');
    if (alphaValEl) alphaValEl.textContent = alphaVal + '%';
}}

function toggleBg() {{
    const enabled = localStorage.getItem(KEY_BG_ENABLED) === 'true';
    localStorage.setItem(KEY_BG_ENABLED, enabled ? 'false' : 'true');
    applyBg();
}}

function applyBgUrl() {{
    const input = document.getElementById('bgUrlInput');
    if (!input) return;
    const url = input.value.trim();
    if (!url || url === DEFAULT_BG_URL) localStorage.removeItem(KEY_BG_URL);
    else localStorage.setItem(KEY_BG_URL, url);
    applyBg();
}}

function resetBgUrl() {{
    localStorage.removeItem(KEY_BG_URL);
    const input = document.getElementById('bgUrlInput');
    if (input) input.value = DEFAULT_BG_URL;
    applyBg();
}}

function applyBlur(val) {{
    let b = parseInt(val);
    if (isNaN(b)) b = DEFAULT_BLUR;
    b = Math.max(0, Math.min(40, b));
    document.documentElement.style.setProperty('--bg-blur', b + 'px');
    const el = document.getElementById('blurVal');
    if (el) el.textContent = b + 'px';
}}

function saveBlur(val) {{
    applyBlur(val);
    localStorage.setItem(KEY_BG_BLUR, parseInt(val));
}}

function applyAlpha(val) {{
    let a = parseInt(val);
    if (isNaN(a)) a = DEFAULT_ALPHA;
    a = Math.max(10, Math.min(100, a));
    document.documentElement.style.setProperty('--container-alpha', (a / 100).toString());
    const el = document.getElementById('alphaVal');
    if (el) el.textContent = a + '%';
}}

function saveAlpha(val) {{
    applyAlpha(val);
    localStorage.setItem(KEY_BG_ALPHA, parseInt(val));
}}

/* ============ 主题 ============ */
function applyTheme() {{
    const isNight = document.body.dataset.theme === 'night';
    const btn = document.getElementById('themeBtn');
    if (btn) btn.textContent = isNight ? '☀️ 日间' : '🌙 夜间';
    const st = document.getElementById('setTheme');
    if (st) st.classList.toggle('on', isNight);
}}

function toggleTheme() {{
    const isNight = document.body.dataset.theme === 'night';
    document.body.dataset.theme = isNight ? '' : 'night';
    localStorage.setItem(KEY_THEME, isNight ? 'day' : 'night');
    applyTheme();
}}

/* ============ 视图 ============ */
function setView(view) {{
    document.body.dataset.view = view;
    localStorage.setItem(KEY_VIEW, view);
    document.getElementById('viewGrid').classList.toggle('active', view === 'grid');
    document.getElementById('viewList').classList.toggle('active', view === 'list');
    const sel = document.getElementById('setView');
    if (sel) sel.value = view;
}}

function applyColumns(cols) {{
    const grid = document.getElementById('albumGrid');
    if (grid) grid.style.gridTemplateColumns = 'repeat(' + cols + ', 1fr)';
}}

function changeColumns(val) {{
    let cols = parseInt(val || document.getElementById('colsSelect').value) || 2;
    cols = Math.max(2, Math.min(6, cols));
    localStorage.setItem(KEY_COLS, cols);
    applyColumns(cols);
    const s1 = document.getElementById('colsSelect'); if (s1) s1.value = cols;
    const s2 = document.getElementById('setCols'); if (s2) s2.value = cols;
}}

/* ============ 封面 ============ */
function applyCoverState() {{
    document.querySelectorAll('.album-card').forEach(card => {{
        const img = card.querySelector('.cover img');
        if (img) img.src = showCover ? (card.getAttribute('data-cover') || DEFAULT_COVER_JS) : DEFAULT_COVER_JS;
    }});
    const el1 = document.getElementById('coverToggle'); if (el1) el1.classList.toggle('on', showCover);
    const el2 = document.getElementById('setCover'); if (el2) el2.classList.toggle('on', showCover);
}}

function toggleCover() {{
    showCover = !showCover;
    localStorage.setItem(KEY_COVER, showCover ? 'true' : 'false');
    applyCoverState();
}}

/* ============ 排序 ============ */
function sortAlbums(val) {{
    const grid = document.getElementById('albumGrid');
    if (!grid) return;
    const sortBy = val || document.getElementById('sortSelect').value;
    localStorage.setItem(KEY_SORT, sortBy);
    const s1 = document.getElementById('sortSelect'); if (s1) s1.value = sortBy;
    const s2 = document.getElementById('setSort'); if (s2) s2.value = sortBy;
    const cards = Array.from(grid.querySelectorAll('.album-card'));
    cards.sort((a, b) => {{
        if (sortBy === 'name') return (a.getAttribute('data-name') || '').localeCompare(b.getAttribute('data-name') || '');
        if (sortBy === 'name-desc') return (b.getAttribute('data-name') || '').localeCompare(a.getAttribute('data-name') || '');
        if (sortBy === 'time') return (parseInt(a.getAttribute('data-modified')) || 0) - (parseInt(b.getAttribute('data-modified')) || 0);
        if (sortBy === 'time-desc') return (parseInt(b.getAttribute('data-modified')) || 0) - (parseInt(a.getAttribute('data-modified')) || 0);
        if (sortBy === 'count') return (parseInt(a.getAttribute('data-count')) || 0) - (parseInt(b.getAttribute('data-count')) || 0);
        if (sortBy === 'count-desc') return (parseInt(b.getAttribute('data-count')) || 0) - (parseInt(a.getAttribute('data-count')) || 0);
        if (sortBy === 'views') return (parseInt(a.getAttribute('data-views')) || 0) - (parseInt(b.getAttribute('data-views')) || 0);
        if (sortBy === 'views-desc') return (parseInt(b.getAttribute('data-views')) || 0) - (parseInt(a.getAttribute('data-views')) || 0);
        return 0;
    }});
    cards.forEach(card => grid.appendChild(card));
}}

/* ============ 全部搜索 ============ */
async function loadAllAlbums() {{
    try {{
        const resp = await fetch('/api/albums');
        const data = await resp.json();
        allAlbumsCache = data.albums || [];
    }} catch (e) {{ console.error(e); }}
}}

function filterAlbums() {{
    const query = document.getElementById('searchInput').value.toLowerCase().trim();
    const grid = document.getElementById('albumGrid');
    const folderGrid = document.getElementById('folderGrid');
    const noResult = document.getElementById('noResult');
    if (query === '') {{
        if (showSearchResults) {{ location.reload(); return; }}
        document.querySelectorAll('.album-card, .folder-card').forEach(c => c.classList.remove('hidden'));
        if (folderGrid) folderGrid.style.display = '';
        noResult.style.display = 'none';
        document.getElementById('visibleCount').textContent = document.querySelectorAll('.album-card').length;
        return;
    }}
    showSearchResults = true;
    if (folderGrid) folderGrid.style.display = 'none';
    const matches = allAlbumsCache.filter(a => a.name.toLowerCase().includes(query));
    if (matches.length === 0) {{
        grid.innerHTML = '';
        noResult.style.display = 'block';
    }} else {{
        grid.innerHTML = matches.map(albumCardHtml).join('');
        noResult.style.display = 'none';
        applyCoverState();
        renderProgress();
        sortAlbums();
    }}
    document.getElementById('visibleCount').textContent = matches.length;
}}

/* ============ 随机 ============ */
async function randomAlbum() {{
    try {{
        const resp = await fetch('/api/random');
        const data = await resp.json();
        if (data.album && data.album.path) {{
            location.href = '/viewer?path=' + encodeURIComponent(data.album.path);
        }} else {{
            alert('暂无漫画');
        }}
    }} catch (e) {{ alert('获取随机漫画失败'); }}
}}

/* ============ 刷新 ============ */
function refreshPage() {{
    location.reload();
}}

/* ============ 历史 ============ */
function toggleHistory(e) {{
    if (e) e.stopPropagation();
    const panel = document.getElementById('historyPanel');
    document.getElementById('statsPanel').classList.remove('show');
    if (panel.classList.contains('show')) {{ panel.classList.remove('show'); }}
    else {{ renderHistoryList(); panel.classList.add('show'); }}
}}

function renderHistoryList() {{
    const list = document.getElementById('historyList');
    let history = [];
    try {{ history = JSON.parse(localStorage.getItem(KEY_HISTORY) || '[]'); }} catch (e) {{}}
    if (history.length === 0) {{
        list.innerHTML = '<div class="history-empty">暂无浏览记录</div>';
    }} else {{
        list.innerHTML = history.map(function(h) {{
            return '<a href="/viewer?path=' + encodeURIComponent(h.path) + '" class="history-item">' + escapeHtml(h.name) + '</a>';
        }}).join('');
    }}
}}

/* ============ 统计 ============ */
async function toggleStats(e) {{
    if (e) e.stopPropagation();
    const panel = document.getElementById('statsPanel');
    document.getElementById('historyPanel').classList.remove('show');
    if (panel.classList.contains('show')) {{ panel.classList.remove('show'); return; }}
    try {{
        const resp = await fetch('/api/stats');
        const data = await resp.json();
        document.getElementById('statAlbums').textContent = data.total_albums || 0;
        document.getElementById('statImages').textContent = data.total_images || 0;
        document.getElementById('statSize').textContent = formatSize(data.total_size || 0);
    }} catch (err) {{ console.error(err); }}
    panel.classList.add('show');
}}

/* ============ 进度 ============ */
function renderProgress() {{
    document.querySelectorAll('.album-card').forEach(card => {{
        const path = card.getAttribute('data-path');
        if (!path) return;
        const key = 'nwebp_progress_' + path;
        try {{
            const data = JSON.parse(localStorage.getItem(key));
            if (data && data.page >= 0 && data.total > 0) {{
                const percent = Math.round((data.page + 1) / data.total * 100);
                const bar = card.querySelector('.progress-bar');
                const fill = card.querySelector('.progress-fill');
                const text = card.querySelector('.progress-text');
                if (bar && fill) {{ bar.style.display = 'block'; fill.style.width = percent + '%'; }}
                if (text) text.textContent = (data.page + 1) + '/' + data.total;
            }}
        }} catch (e) {{}}
    }});
}}

/* ============ 设置 ============ */
function openSettings() {{
    document.getElementById('settingsOverlay').classList.add('show');
    document.body.style.overflow = 'hidden';
    document.getElementById('setTheme').classList.toggle('on', document.body.dataset.theme === 'night');
    document.getElementById('setCover').classList.toggle('on', showCover);
    document.getElementById('setCols').value = localStorage.getItem(KEY_COLS) || 2;
    document.getElementById('setView').value = document.body.dataset.view || 'grid';
    document.getElementById('setSort').value = document.getElementById('sortSelect').value;
    applyBg();
}}

function closeSettings() {{
    document.getElementById('settingsOverlay').classList.remove('show');
    document.body.style.overflow = '';
}}

function selectAllClear() {{
    document.querySelectorAll('#clearOptions input[type="checkbox"]').forEach(cb => cb.checked = true);
}}
function unselectAllClear() {{
    document.querySelectorAll('#clearOptions input[type="checkbox"]').forEach(cb => cb.checked = false);
}}

function clearSelected() {{
    const selected = document.querySelectorAll('#clearOptions input[type="checkbox"]:checked');
    if (selected.length === 0) {{ alert('请先选择要清除的项目'); return; }}
    if (!confirm('确定清除选中的 ' + selected.length + ' 项？')) return;
    const map = {{
        theme: ['nwebp_theme'],
        view: ['nwebp_view'],
        columns: ['nwebp_columns'],
        sort: ['nwebp_sort'],
        cover: ['nwebp_show_cover'],
        history: ['nwebp_history'],
        bg: ['nwebp_bg_enabled', 'nwebp_bg_url', 'nwebp_bg_blur', 'nwebp_container_alpha'],
        viewer: ['nwebp_viewer_mode','nwebp_viewer_arrows','nwebp_viewer_rtl','nwebp_viewer_click','nwebp_viewer_touch'],
        popup: ['nwebp_popup_shown']
    }};
    selected.forEach(cb => {{
        const type = cb.dataset.clear;
        if (type === 'progress') {{
            const toRemove = [];
            for (let i = 0; i < localStorage.length; i++) {{
                const k = localStorage.key(i);
                if (k && k.startsWith('nwebp_progress_')) toRemove.push(k);
            }}
            toRemove.forEach(k => localStorage.removeItem(k));
        }} else if (map[type]) {{
            map[type].forEach(k => localStorage.removeItem(k));
        }}
    }});
    alert('已清除选中项，即将刷新页面');
    location.reload();
}}

function clearAll() {{
    if (!confirm('确定清除全部缓存？')) return;
    const keys = [];
    for (let i = 0; i < localStorage.length; i++) {{
        const k = localStorage.key(i);
        if (k && k.startsWith('nwebp_')) keys.push(k);
    }}
    keys.forEach(k => localStorage.removeItem(k));
    alert('缓存已清除，即将刷新页面');
    location.reload();
}}

/* ============ 其它 ============ */
function scrollToTop() {{ window.scrollTo({{ top: 0, behavior: 'smooth' }}); }}

window.addEventListener('scroll', function() {{
    const btn = document.getElementById('backTop');
    if (btn) btn.classList.toggle('show', window.scrollY > 300);
}});

document.addEventListener('click', function(e) {{
    const hp = document.getElementById('historyPanel');
    const sp = document.getElementById('statsPanel');
    const hb = document.getElementById('historyBtn');
    const sb = document.getElementById('statsBtn');
    if (hp && hb && hp.classList.contains('show') && !hp.contains(e.target) && !hb.contains(e.target)) hp.classList.remove('show');
    if (sp && sb && sp.classList.contains('show') && !sp.contains(e.target) && !sb.contains(e.target)) sp.classList.remove('show');
}});

/* ============ 初始化 ============ */
(function init() {{
    const savedTheme = localStorage.getItem(KEY_THEME);
    if (savedTheme === 'night') document.body.dataset.theme = 'night';
    applyTheme();
    applyBg();
    setView(localStorage.getItem(KEY_VIEW) || 'grid');
    const cols = localStorage.getItem(KEY_COLS) || 2;
    document.getElementById('colsSelect').value = cols;
    document.getElementById('setCols').value = cols;
    applyColumns(cols);
    applyCoverState();
    const sort = localStorage.getItem(KEY_SORT) || 'name';
    document.getElementById('sortSelect').value = sort;
    document.getElementById('setSort').value = sort;
}})();

loadAllAlbums();
applyCoverState();
renderProgress();
sortAlbums();

window.addEventListener('pageshow', (e) => {{
    if (e.persisted) {{ loadAllAlbums(); applyCoverState(); renderProgress(); sortAlbums(); }}
}});

{popup_js}
</script>
</body>
</html>
"#,
        title = config.title,
        subtitle = config.subtitle,
        footer = config.footer,
        accent = accent_color,
        total = total,
        empty_hint = empty_hint,
        cards = album_cards,
        folder_section = folder_section,
        breadcrumbs_html = breadcrumbs_html,
        popup_css = popup_css,
        popup_html = popup_html,
        popup_js = popup_js,
    )
}