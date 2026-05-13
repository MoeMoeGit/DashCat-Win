# DashCat for Windows

Windows 11 系统托盘监控工具，复刻自 macOS 版 DashCat。

## 功能

- **系统监控**：CPU/内存使用率实时显示
- **睡眠阻止**：防止系统休眠或显示器关闭
- **开机自启**：支持注册表自启动
- **猫咪动画**：托盘图标显示动态猫咪

## 下载

前往 [Releases](https://github.com/MoeMoeGit/DashCat-Win/releases) 页面下载最新版本。

## 构建

需要 Rust 和 mingw-w64 交叉编译环境：

```bash
# 安装目标
rustup target add x86_64-pc-windows-gnu

# 构建
cargo build --release --target x86_64-pc-windows-gnu
```

## 技术栈

- Rust + windows-rs (纯 Win32 API)
- PDH 计数器（CPU监控）
- GlobalMemoryStatusEx（内存监控）
- SetThreadExecutionState（睡眠阻止）

## 许可证

MIT