# 项目架构

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                     DashCat-Win (单进程)                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Tray UI   │  │  Monitor    │  │  Clipboard Manager  │  │
│  │  (Win32)    │──│   (PDH)     │──│  (Win32 + SQLite)   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│         │                │                    │              │
│         ▼                ▼                    ▼              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Config (JSON/Registry)                  │    │
│  └─────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                    Windows System APIs                       │
│  Shell_NotifyIcon | PDH | Clipboard | Power | Hooks         │
└─────────────────────────────────────────────────────────────┘
```

## 目录结构

```
dashcat-win/
├── src/
│   ├── main.rs              # 入口点、初始化
│   ├── tray/                # 系统托盘模块
│   │   ├── mod.rs
│   │   ├── icon.rs          # 图标渲染、动画
│   │   └── menu.rs          # 右键菜单
│   │   └── panel.rs         # 剪贴板面板窗口
│   ├── monitor/             # 系统监控模块
│   │   ├── mod.rs
│   │   ├── cpu.rs           # CPU 采集
│   │   └── memory.rs        # 内存采集
│   ├── clipboard/           # 剪贴板管理模块
│   │   ├── mod.rs
│   │   ├── manager.rs       # 剪贴板监控、存储
│   │   ├── db.rs            # SQLite 操作
│   │   └── history.rs       # 历史记录结构
│   ├── power/               # 防休眠模块
│   │   ├── mod.rs
│   │   └── caffeine.rs      # SetThreadExecutionState
│   ├── scroll/              # 鼠标滚轮反转模块
│   │   ├── mod.rs
│   │   └── hook.rs          # WH_MOUSE_LL 钩子
│   ├── config/              # 配置管理
│   │   ├── mod.rs
│   │   ├── settings.rs      # 用户设置
│   │   └── locale.rs        # 多语言
│   └── assets/              # 内嵌资源
│       ├── cat_0.png        # 猫咪动画帧
│       ├── cat_1.png
│       ├── cat_2.png
│       ├── cat_3.png
│       ├── cat_4.png
│       └── locales.json     # 多语言翻译
├── project-log/             # 开发知识库
├── Cargo.toml
├── build.rs                 # 构建脚本（资源嵌入）
└── README.md
```

## 关键技术决策

### 决策 1：纯 Win32 API 而非 UI 框架

- **选择**：直接使用 Win32 API (windows-rs crate)
- **备选方案**：
  1. WinUI 3 / XAML — 需要引入 Windows App SDK，二进制膨胀
  2. imgui — 游戏风格 UI，不符合原生外观
  3. Qt / GTK — 跨平台但引入重型依赖
- **原因**：
  - 目标是"最省资源、启动最快"
  - 系统托盘应用 UI 简单，Win32 API 足够
  - windows-rs 提供安全的 Rust 封装
- **参考**：详见 `10-planning-log.md` ADR-001

### 决策 2：PNG 动画帧内嵌二进制

- **选择**：使用 `include_bytes!` 将 PNG 文件编译进二进制
- **备选方案**：
  1. 外部文件读取 — 需要处理路径问题
  2. 系统图标 — 无法实现猫咪动画
- **原因**：
  - 确保单文件分发，无需额外资源文件
  - macOS 版同样内嵌图片资源

### 决策 3：交叉编译而非原生构建

- **选择**：在 Linux 上使用 `x86_64-pc-windows-gnu` 交叉编译
- **备选方案**：
  1. Windows 原生编译 — 需要 Windows 环境
  2. GitHub Actions — 自动化但增加依赖
- **原因**：
  - CatMeter-Windows 已验证交叉编译可行
  - 开发环境在 Linux，无需切换系统
- **参考**：详见 `10-planning-log.md` ADR-002

## 依赖关系

| 依赖 | 版本 | 用途 |
|------|------|------|
| windows | 0.54 | Win32 API 封装 |
| rusqlite | 0.31 | SQLite 数据库 |
| png | 0.17 | PNG 解码 |
| serde | 1.0 | 配置/多语言序列化 |
| chrono | 0.4 | 时间处理 |
| uuid | 1.0 | 图片文件命名 |

## 变更记录

| 日期 | 变更内容 | 原因 |
|------|----------|------|
| 2026-05-13 | 初始架构设计 | 项目启动 |