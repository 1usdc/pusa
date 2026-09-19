# Pusa

Pusa 是一套 Agent 控制台，同时提供**桌面端**与 **Web** 两种形态：在同一套界面里对话、管理技能，并在桌面端打开本机工作区。

插件市场对接 [Open VSX](https://open-vsx.org/) 开放 API，可安装 VS Code 兼容扩展（优先用于语法高亮与编程语言支持）。

## 能做什么

- **对话**：与 Agent 聊天，配置模型与密钥
- **工作区（桌面）**：打开本机项目，浏览、编辑文件，使用内置终端
- **技能库**：浏览、安装并装备 Agent 技能
- **插件市场（桌面）**：从 Open VSX 搜索、安装扩展到工作区 `extensions/`
- **内置浏览器**：在应用内打开网页

Web 端侧重对话与技能；本机文件、终端和扩展安装请使用桌面应用。

## 环境要求

从源码运行需要：

- [Rust](https://www.rust-lang.org/)
- [just](https://github.com/casey/just)
- [Dioxus CLI](https://dioxuslabs.com/)（`dx`）

Windows 编译桌面端还需 MSVC C++ 生成工具（提供 `link.exe`）：

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

## 运行

### 桌面端

```bash
just desktop
```

### Web

分别启动前端与 API，然后在浏览器打开 `http://127.0.0.1:8080/`：

```bash
just web
just server
```

## 仓库结构

公开层面主要包含：

| 路径 | 说明 |
|------|------|
| `desktop/` | 桌面端入口 |
| `web/` | Web 端入口 |
| `server/` | Web 所用 HTTP API |
| `ui/` | 桌面与 Web 共用界面 |
| `protocol/` | 前后端共用的数据类型 |
| `extensions/` | 工作区扩展（含从 Open VSX 安装的包） |
| `skills/` | Agent 技能包 |

界面由 Rust 与 [Dioxus](https://dioxuslabs.com/) 实现。
