# extensions/

存放 **VS Code 兼容扩展**（含从 Open VSX 安装的 `.vsix`）与内部插件脚手架。

与 `skills/`（Agent 技能 Markdown 包）不同：本目录用于编辑器/产品侧扩展能力。市场默认数据源为 [Open VSX](https://open-vsx.org/) 开放 API，优先推荐语法高亮 / Programming Languages 类扩展。

## 约定

- 每个扩展一个子目录，目录名一般为 `publisher.name`
- Open VSX 安装会写入 `.pusa-plugin.json`，并在可解析时生成 `.pusa-grammars.json`（`contributes.grammars` / `languages` 摘要）
- 打开文件时编辑器读取 `.pusa-grammars.json`（或 `package.json`）里的 `contributes.grammars` / `languages`，按扩展名或语言 id 加载 TextMate（JSON / plist）并套到现有 `ac-syn-*` 高亮；无匹配、文件过大或语法无法解析时回退 `ui/src/shell/syntax.rs` 内置分词
- 不加入 Cargo workspace（与 `skills/` 同为内容目录）

相关目录：

| 目录 | 用途 |
|------|------|
| `skills/` | Agent 技能（`SKILL.md`） |
| `extensions/` | VS Code / Open VSX 扩展与内部插件 |
| `applications/` | 本地脚手架应用（智能 UI 等） |
