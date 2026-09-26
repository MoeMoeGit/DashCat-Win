# 数据

依据 `src/config/settings.rs`：设置保存为 `%APPDATA%\DashCat\settings.json`，字段涵盖监控/显示/休眠模式、图片开关、滚轮开关、自启、语言与历史上限。未接通功能对应字段是预留。

依据 `src/clipboard/db.rs`：实际表为 `clipboard`，字段 id、content BLOB、content_type、is_pinned、created_at INTEGER（Unix 秒）、preview；索引 idx_preview。旧 clipboard_history、image_path、REAL 时间戳及 WAL 的描述不是当前实现。

ClipboardManager 使用传入 data_dir 下的 clipboard.db。图片序列化未完成，不能承诺旧设计中的 Images 目录或容量清理。当前仅 CREATE TABLE IF NOT EXISTS，无版本迁移框架。
