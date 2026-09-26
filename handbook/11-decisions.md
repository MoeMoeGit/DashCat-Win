# 决策

选择 Win32 而非 Qt/WinUI，降低运行依赖并保持原生菜单。GNU 交叉编译用于开发，现有 CI 同时提供 Windows MSVC/MSIX 路径；旧“放弃 CI”理由不再适用。

动画通过 include_bytes! 内嵌，保持单文件分发。旧 SQLite 图片文件方案未落地，实际数据契约见 05。Windows-rs 不消除 unsafe 边界，仍需核对系统调用和资源释放。
