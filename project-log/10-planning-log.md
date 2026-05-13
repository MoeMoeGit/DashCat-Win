# 决策记录

---

### ADR-001 [2026-05-13] UI 技术选型：纯 Win32 API

**状态**：已采用

**替代关系**：无

**背景与需求**：
- 目标是"最省资源、启动最快、最稳定"
- 系统托盘应用 UI 简单（图标 + 菜单 + 弹出面板）
- 需要与 macOS 版体验一致，但 Windows 原生外观

**采用的方案**：使用 windows-rs crate 直接调用 Win32 API

**备选方案**：

1. WinUI 3 / XAML Islands
   - 优点：现代 UI、声明式开发
   - 缺点：需要 Windows App SDK (~50MB)、二进制膨胀、冷启动慢
   - 放弃原因：违反"最省资源"原则

2. imgui-rs
   - 优点：轻量、即时模式、游戏风格
   - 缺点：不符合 Windows 原生外观、需要自定义渲染循环
   - 放弃原因：体验与 macOS 版不一致

3. Qt / GTK
   - 优点：跨平台成熟方案
   - 缺点：引入重型依赖、二进制 > 10MB
   - 放弃原因：违反"零依赖"原则

**决策依据**：
- CatMeter-Windows 项目已验证 windows-rs 可行性
- 系统托盘应用只需 Shell_NotifyIcon + 简单窗口，Win32 足够
- windows-rs 提供安全 Rust 封装，避免 raw pointer 问题

**改动范围**：
- Cargo.toml 依赖 windows = "0.54"
- src/tray/ 模块使用 Shell_NotifyIcon 等 API

---

### ADR-002 [2026-05-13] 构建方式：交叉编译

**状态**：已采用

**替代关系**：无

**背景与需求**：
- 开发环境是 Ubuntu Linux
- 目标平台是 Windows 11 x86_64
- 需要能在 Linux 上构建 Windows 可执行文件

**采用的方案**：使用 `x86_64-pc-windows-gnu` target 交叉编译

**备选方案**：

1. GitHub Actions CI 构建
   - 优点：自动化、稳定 Windows 环境
   - 缺点：依赖外部服务、调试不方便
   - 放弃原因：开发阶段需要快速迭代，CI 延迟太高

2. Windows 虚拟机原生编译
   - 优点：最兼容、调试方便
   - 缺点：需要切换系统、资源占用大
   - 放弃原因：开发效率低

3. Windows 物理机远程编译
   - 优点：原生环境
   - 缺点：需要额外硬件
   - 放弃原因：资源浪费

**决策依据**：
- CatMeter-Windows 已成功使用交叉编译
- windows-rs 0.54 支持 GNU target
- 开发阶段可快速迭代，稳定后可添加 CI

**改动范围**：
- 安装 mingw-w64 工具链
- `rustup target add x86_64-pc-windows-gnu`
- Cargo.toml 配置 linker

---

### ADR-003 [2026-05-13] 动画帧存储：二进制内嵌

**状态**：已采用

**替代关系**：无

**背景与需求**：
- 猫咪动画需要 5 帧 PNG 图片
- macOS 版使用 Asset Catalog 内嵌
- Windows 版需要同样方便的分发方式

**采用的方案**：使用 `include_bytes!` 将 PNG 编译进二进制

**备选方案**：

1. 外部文件读取
   - 优点：可替换图片
   - 缺点：需要处理安装路径、分发多文件
   - 放弃原因：违反"单文件分发"目标

2. Base64 编码字符串
   - 优点：纯文本存储
   - 缺点：编译时间增加、二进制膨胀 ~33%
   - 放弃原因：不必要的中间转换

**决策依据**：
- 单文件分发是桌面工具最佳实践
- 5 张 PNG 总大小 < 50KB，内嵌影响极小
- macOS 版同样内嵌资源

**改动范围**：
- src/assets/ 目录存放 PNG
- build.rs 可选（资源预处理）
- 代码中使用 `include_bytes!("../assets/cat_0.png")`

---

<!-- 新决策记录追加在此处 -->