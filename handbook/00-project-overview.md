# 项目概述

Windows 11 x64 的轻量托盘系统监控工具，源于 macOS DashCat 的使用体验。已实现猫咪动画、CPU/内存显示、防休眠和开机启动；剪贴板模块存在但尚未接入用户主流程，鼠标反转与多语言仍是目标。

沿用 Rust 2021、windows-rs 0.54、serde/serde_json、png、rusqlite 0.31 和 clipboard-rs 0.3；确切版本以 Cargo.lock 为准。Cargo.toml 当前版本 0.4.0，未声明 rust-version；旧文档中的 Rust 最低版本不能当作验证结论。

无账号和云同步。原定低内存、小包体、快速启动为目标，本轮没有性能实测。
