# Spec Delta: Data Storage

## ADDED Requirements

### Requirement: SQLite 数据库初始化

系统 MUST 使用 SQLite 作为本地存储,并 SHALL 在首次启动时自动创建数据库和表结构。

#### Scenario: 首次启动创建数据库

- **WHEN** 应用首次启动
- **THEN** 应该在应用数据目录创建 `lingcode.db` 文件
- **AND** 应该执行初始化 SQL 脚本创建表结构

#### Scenario: 数据库路径

- **WHEN** 应用运行在 macOS
- **THEN** 数据库文件应该位于 `~/Library/Application Support/com.lingcode.app/lingcode.db`
- **AND** 应该自动创建父目录

#### Scenario: 数据库 Schema 验证

- **WHEN** 应用启动时
- **THEN** 应该验证数据库 schema 版本
- **AND** 如果版本不匹配,应该执行迁移脚本

---

### Requirement: 设置数据持久化

系统 MUST 将用户设置存储到数据库,并 SHALL 支持读取、更新和重置操作。

#### Scenario: 保存设置

- **WHEN** 用户修改快捷键为 `Cmd+Shift+Space`
- **THEN** 应该调用 Tauri Command `save_setting("hotkey", "Cmd+Shift+Space")`
- **AND** 设置应该保存到 `settings` 表

#### Scenario: 读取设置

- **WHEN** 应用启动时
- **THEN** 应该从数据库加载所有设置
- **AND** 如果设置不存在,应该使用默认值

#### Scenario: 重置设置

- **WHEN** 用户点击"恢复默认设置"
- **THEN** 应该删除所有自定义设置
- **AND** 应用应该使用默认值

---

### Requirement: 历史记录管理

系统 MUST 将语音转录历史记录存储到数据库,并 SHALL 支持查询、搜索和删除操作。

#### Scenario: 保存转录记录

- **WHEN** 语音转录完成,内容为"你好世界"
- **THEN** 应该创建一条记录到 `transcriptions` 表
- **AND** 记录应该包含:文本内容、音频时长、模型版本、语言、时间戳、应用上下文

#### Scenario: 查询历史记录

- **WHEN** 用户打开历史记录窗口
- **THEN** 应该按时间倒序查询所有记录
- **AND** 每页显示 50 条,支持分页加载

#### Scenario: 搜索历史记录

- **WHEN** 用户在搜索框输入"你好"
- **THEN** 应该执行 SQL `LIKE` 查询
- **AND** 只显示包含"你好"的记录

#### Scenario: 删除单条记录

- **WHEN** 用户点击某条记录的删除按钮
- **THEN** 应该显示确认对话框
- **AND** 确认后从数据库删除该记录

#### Scenario: 清空历史记录

- **WHEN** 用户点击"清空所有历史"
- **THEN** 应该显示警告对话框
- **AND** 确认后删除 `transcriptions` 表的所有记录

---

### Requirement: 数据迁移机制

系统 MUST 支持数据库 schema 版本管理和自动迁移。

#### Scenario: Schema 版本检测

- **WHEN** 应用启动时
- **THEN** 应该读取 `settings` 表中的 `schema_version`
- **AND** 与当前代码的 schema 版本对比

#### Scenario: 自动迁移

- **WHEN** 检测到 schema 版本低于当前版本
- **THEN** 应该按顺序执行迁移脚本 (例如 `migration_v1_to_v2.sql`)
- **AND** 迁移成功后更新 `schema_version`

#### Scenario: 迁移失败回滚

- **WHEN** 迁移脚本执行失败
- **THEN** 应该回滚事务
- **AND** 显示错误提示,阻止应用启动
- **AND** 建议用户备份数据库并报告问题

---

### Requirement: 数据导出和导入

系统 MUST 支持导出历史记录为 JSON 或 CSV 格式,并 SHALL 支持导入备份数据。

#### Scenario: 导出为 JSON

- **WHEN** 用户点击"导出历史记录" → "JSON 格式"
- **THEN** 应该打开文件保存对话框
- **AND** 导出所有历史记录为 JSON 数组
- **AND** 文件名默认为 `lingcode-history-YYYY-MM-DD.json`

#### Scenario: 导出为 CSV

- **WHEN** 用户点击"导出历史记录" → "CSV 格式"
- **THEN** 应该导出为 CSV 文件,包含表头
- **AND** 字段为:时间、文本、时长、语言、应用

#### Scenario: 导入备份数据

- **WHEN** 用户点击"导入历史记录"并选择 JSON 文件
- **THEN** 应该读取文件内容
- **AND** 验证数据格式是否正确
- **AND** 将记录插入数据库(跳过重复记录)

---

### Requirement: 数据库性能优化

系统 MUST 优化数据库查询性能,以确保历史记录达到 10000 条时仍然流畅。

#### Scenario: 索引优化

- **WHEN** 数据库初始化时
- **THEN** 应该为 `transcriptions.created_at` 创建降序索引
- **AND** 应该为 `transcriptions.text` 创建全文索引 (FTS5)

#### Scenario: 分页查询

- **WHEN** 查询历史记录
- **THEN** 应该使用 `LIMIT` 和 `OFFSET` 实现分页
- **AND** 单次查询时间应小于 100ms

#### Scenario: 搜索性能

- **WHEN** 用户搜索关键词
- **THEN** 应该使用 FTS5 全文索引
- **AND** 搜索 10000 条记录的响应时间应小于 200ms
