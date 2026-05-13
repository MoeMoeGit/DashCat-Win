# DashCat-Win

Windows 版系统托盘猫咪工具 — 剪贴板历史、系统监控、防休眠、鼠标滚轮反转。

**原版仓库**: [DashCat (macOS)](https://github.com/vivalucas/DashCat)

---

## 功能

- **系统监控** — CPU/内存实时监控，猫咪动画速度反映负载
- **剪贴板管理** — 历史记录、搜索、Pin 固定、图片支持
- **防止休眠** — 三档控制：关闭/阻止系统休眠/阻止屏幕关闭
- **鼠标滚轮反转** — 仅对鼠标生效，触控板保持自然滚动
- **多语言支持** — 11 种语言界面
- **开机启动** — 注册表自启动

## 系统要求

- Windows 11 x86_64
- Windows 10 21H2+ (可能兼容，非官方支持)

## 安装

从 [Releases](../../releases) 下载最新版本。

## 编译

### Linux 交叉编译

```bash
# 安装 mingw-w64
sudo apt install mingw-w64

# 添加 Rust target
rustup target add x86_64-pc-windows-gnu

# 编译
cargo build --release --target x86_64-pc-windows-gnu
```

### Windows 原生编译

```bash
cargo build --release
```

## 开发文档

开发者请查看 `project-log/` 目录。

---

MIT License
