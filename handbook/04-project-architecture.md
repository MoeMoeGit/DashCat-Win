# 架构

`src/main.rs` 加载 Settings 后调用 TrayApp；`src/tray/` 负责 Win32 消息循环、菜单与图标；`src/monitor/` 采集 CPU/内存；`src/power/` 管理休眠；`src/config/` 管理 JSON 设置与自启注册表。

`src/clipboard/` 已提供 ClipboardManager 和 ClipboardDb，但当前 TrayApp 尚未使用。不能据此宣称剪贴板 UI 已完成。`src/assets/` 保存五帧动画，`assets/icons/` 保存发行图标。
