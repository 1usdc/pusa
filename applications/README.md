# applications/

存放**外部应用**的安装目录。

与 `skills/`（Agent 技能）和 `extensions/`（内部插件）不同：本目录用于第三方或独立可安装的应用包。桌面端「应用市场」会将选中的项目以 `git clone --depth 1` 下载到此目录的子文件夹，并写入 `.pusa-plugin.json` 元数据。

## 约定（暂定）

- 每个应用一个子目录
- 不加入 Cargo workspace（与 `skills/` 同为内容目录）
- 安装与加载逻辑由应用侧后续接入
- 桌面端打开「我的应用」时，会自动扫描尚未缓存智能 UI 的子目录，将结果写入各应用下的 `.pusa-smart-ui.json`（下载安装后也会触发）

相关目录：

| 目录 | 用途 |
|------|------|
| `skills/` | Agent 技能（`SKILL.md`） |
| `extensions/` | 内部插件 |
| `applications/` | 外部应用 |

| 文件 | 用途 |
|------|------|
| `.pusa-plugin.json` | 安装元数据（名称、来源、git 等） |
| `.pusa-smart-ui.json` | 智能 UI 缓存（可执行动作面板） |
