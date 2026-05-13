# 数据库设计

## 数据库选型

| 项目 | 选择 | 说明 |
|------|------|------|
| 数据库类型 | SQLite | 单文件、零配置、跨平台兼容 |
| 版本 | 3.x | rusqlite 默认版本 |
| ORM / 驱动 | rusqlite 0.31+ | Rust 原生 SQLite 绑定 |

## ER 关系概览

```
clipboard_history (单表，无外键关系)
```

## 表设计

### clipboard_history 剪贴板历史表

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PRIMARY KEY, AUTOINCREMENT | 自增主键 |
| content | TEXT | NULL | 文本内容（最大 10000 字符） |
| image_path | TEXT | NULL | 图片文件路径（相对路径） |
| source_app | TEXT | NOT NULL, DEFAULT '' | 来源应用名 |
| is_pinned | INTEGER | NOT NULL, DEFAULT 0 | 是否固定 (0/1) |
| created_at | REAL | NOT NULL | 创建时间戳 (Unix timestamp) |

### 索引

| 表 | 索引名 | 字段 | 说明 |
|----|--------|------|------|
| clipboard_history | idx_created_at | created_at | 按时间排序查询优化 |
| clipboard_history | idx_pinned | is_pinned | 固定项查询优化 |

### 设计决策

1. **单表设计**：剪贴板历史结构简单，无需关联查询，单表足够
2. **图片存储**：图片存文件系统，数据库只存路径，避免 BLOB 性能问题
3. **时间戳用 REAL**：与 macOS 版保持一致，使用 Unix timestamp 浮点数
4. **WAL 模式**：启用 Write-Ahead Logging 提高并发性能

## 数据存储路径

```
%APPDATA%\DashCat\
├── clipboard.db      # SQLite 数据库
├── clipboard.db-wal  # WAL 文件
└── Images\           # 图片存储目录
    ├── {uuid}.jpg
    └── {uuid}_thumb.jpg
```

## 变更记录

| 日期 | 变更内容 | 原因 |
|------|----------|------|
| 2026-05-13 | 初始数据库设计 | 项目启动 |
