# 开发日志

---

## 2026-05-13（项目初始化）

**触发原因**：用户要求将 macOS DashCat 移植到 Windows 11，使用 Rust + windows-rs 方案。

**修改内容**：
1. `project-log/` — 创建开发知识库，按规范填写所有文档
2. `00-project-overview.md` — 项目背景、技术栈、约束定义
3. `01-function-design.md` — 5 个核心模块功能拆解
4. `02-database-design.md` — SQLite 剪贴板历史表设计
5. `04-project-architecture.md` — 目录结构、技术决策
6. `05-current-status.md` — 当前进度、待办清单
7. `10-planning-log.md` — ADR-001/002/003 决策记录
8. `Cargo.toml` — Rust 项目配置，依赖 windows-rs 0.54, rusqlite, png 等
9. `src/main.rs` — 入口点
10. `src/tray/` — 托盘模块（图标渲染、右键菜单）
11. `src/monitor/` — 系统监控模块（CPU/内存）
12. `src/clipboard/` — 剪贴板管理模块（SQLite 存储）
13. `src/power/` — 防休眠模块（SetThreadExecutionState）
14. `src/scroll/` — 鼠标滚轮反转模块（WH_MOUSE_LL 钩子）
15. `src/assets/` — 复制猫咪动画帧 PNG (5 帧)
16. `.gitignore` — Git 忽略配置
17. `README.md` — 用户文档
18. Git 仓库初始化

**遇到的问题**：
- 无（项目初始化阶段）

**解决方式**：
- 直接复制 macOS 版的设计思路，映射到 Windows API

**验证方式**：
- 目录结构检查、文件完整性检查

**验证结果**：
- 通过，项目骨架完整，模块划分清晰

---

## 2026-05-13（Rust 项目骨架创建）

**触发原因**：文档完成，开始代码开发。

**修改内容**：
1. `Cargo.toml` — 初始化 Rust 项目，配置依赖
2. `src/main.rs` — 入口点
3. `src/tray/mod.rs` — 托盘模块骨架
4. `src/assets/` — 复制猫咪动画帧 PNG

**遇到的问题**：
- 待记录

**解决方式**：
- 待记录

**验证方式**：
- 待记录

**验证结果**：
- 待记录

---

<!-- 新记录追加在上方分隔线之后、旧记录之前 -->