# nwebp

轻量级漫画图片在线浏览服务器，参考 [nweb](https://github.com/nasyt233/nweb) 的极简风格，用 Rust 编写。

## 特性

- 🚀 **高性能**：基于 Rust + axum + tokio 异步运行时
- 📖 **翻页阅读**：支持左右翻页、键盘快捷键、触屏滑动
- 📜 **滚动模式**：一键切换长图滚动浏览
- 🖼️ **自动扫描**：自动识别目录下的漫画本子（含图片的子目录）
- 📱 **移动端适配**：触屏滑动翻页，响应式布局
- ⚡ **图片预加载**：提前加载前后页，翻页流畅无卡顿
- 🔒 **路径安全**：防止目录穿越攻击
- 📦 **单二进制**：编译后无依赖，直接运行

## 快速开始

### 编译

```bash
# 需要 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 编译
cargo build --release

# 二进制在 target/release/nwebp
```

### 使用

```bash
# 基本用法：nwebp [漫画根目录] [端口]
./nwebp ./comics 8080

# 指定监听地址
./nwebp ./comics 8080 --host 127.0.0.1

# 默认当前目录 + 8080端口
./nwebp
```

然后浏览器访问 `http://localhost:8080`

### 目录结构要求

将每本漫画放在独立的子文件夹中：

```
comics/
├── 本子A/
│   ├── 00001.jpg
│   ├── 00002.jpg
│   └── ...
├── 本子B/
│   ├── 001.png
│   ├── 002.png
│   └── ...
└── 子目录/
    └── 本子C/
        ├── page1.webp
        └── ...
```

支持的图片格式：JPG、PNG、WebP、GIF、BMP

## 操作说明

### 翻页模式（默认）

| 操作 | 功能 |
|------|------|
| ← / A | 上一页 |
| → / D / 空格 | 下一页 |
| Home | 第一页 |
| End | 最后一页 |
| 点击屏幕左 1/3 | 上一页 |
| 点击屏幕右 1/3 | 下一页 |
| 触屏左滑 | 下一页 |
| 触屏右滑 | 上一页 |

### 滚动模式

点击顶部「翻页模式」按钮切换为滚动模式，所有图片纵向排列，上下滑动浏览。

## API

| 接口 | 说明 |
|------|------|
| `GET /api/albums` | 获取所有本子列表 |
| `GET /api/album/{path}` | 获取指定本子的图片列表 |
| `GET /raw/{path}` | 获取原始图片文件 |

## 与 jmcomic 配合

nwebp 可直接读取 jmcomic 下载的目录结构：

```bash
# jmcomic 下载到 ./download 目录
jmcomic --option config.yml <album_id>

# 用 nwebp 直接浏览
nwebp ./download 8080
```

## 项目结构

```
nwebp/
├── Cargo.toml          # 依赖配置
├── src/
│   ├── main.rs         # 入口 + CLI 参数
│   ├── scanner.rs      # 目录扫描 + 图片识别
│   ├── handler.rs      # HTTP 请求处理
│   └── template.rs     # HTML 模板（首页 + 阅读页）
└── README.md
```

## License

MIT
