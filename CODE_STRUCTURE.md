# Pusa 代码结构

本文梳理仓库的模块边界、依赖关系与主要目录职责，便于新人上手与后续改动定位。

## 1. 总览

Pusa 是一套基于 **Rust + Dioxus 0.7** 的 Agent 控制台，同时提供：

| 形态 | Crate | 说明 |
|------|-------|------|
| Web | `web` | WASM 前端（`dx serve`） |
| Desktop | `desktop` | 系统 WebView / wry 桌面端 |
| API | `server` | Axum HTTP + SSE，供 Web 调用 |
| 共享运行时 façade | `shared` | 开源薄封装，动态加载 `libpusa_core` |
| 闭源核心 | `pusa-core`（gitignore） | cdylib **源码**不入库；预编译库在 `vendor/pusa-core/`（开源） |
| 协议 | `protocol` | 前后端共用的 DTO / SSE 类型 |
| UI | `ui` | Dioxus 界面（Web / Desktop 共用） |

```text
┌─────────────┐     ┌─────────────┐
│  web (WASM) │     │   desktop   │
│  dx serve   │     │  wry/native │
└──────┬──────┘     └──────┬──────┘
       │                   │
       │  HTTP/SSE         │ 进程内调 shared
       ▼                   ▼
┌─────────────┐     ┌─────────────┐
│   server    │────▶│   shared    │
└─────────────┘     └──────┬──────┘
       ▲                   │
       └──── protocol ─────┘
              ▲
              │
           ┌──┴──┐
           │  ui │  (被 web / desktop 引用)
           └─────┘
```

Workspace 成员（根 `Cargo.toml`）：

`protocol` · `shared` · `ui` · `web` · `desktop` · `server` · `test`（`pusa-core` 独立 workspace，不进根 members）

---

## 2. 目录树（仓库根）

```text
pusa/
├── protocol/          # 共享 JSON/SSE 类型
├── shared/            # 开源 façade（JSON-RPC → cdylib）
├── pusa-core/         # 闭源源码（gitignore）；打包为 libpusa_core
├── vendor/pusa-core/  # 预编译 cdylib（开源；按 target triple）
├── ui/                # Dioxus UI（壳层 + 聊天 + 桌面/Web 适配）
├── web/               # Web 入口
├── desktop/           # 桌面入口与资源
├── server/            # HTTP API 入口
├── applications/       # 外部应用安装目录（git clone / 本地脚手架）
├── extensions/         # 内部插件目录（约定中）
├── skills/            # Agent 技能包（SKILL.md）
├── scripts/           # just 调用的 shell 脚本
├── docker/            # 容器与环境变量
├── data/              # 本地 SQLite 等运行数据
├── patches/           # Cargo patch（dioxus / blitz 等）
├── dist/              # 构建产物
├── justfile           # 常用命令入口
├── Dioxus.toml        # Dioxus 工程配置
├── AGENTS.md          # Dioxus 0.7 编码约定（给 Agent 读）
└── README.md          # 启动与发版说明
```

内容目录与代码 crate 的分工：

| 目录 | 类型 | 用途 |
|------|------|------|
| `skills/` | 内容 | Agent 可加载技能 |
| `extensions/` | 内容 | 产品内部插件 |
| `applications/` | 内容 | 外部/本地应用；含 `.pusa-plugin.json`、`.pusa-smart-ui.json` |

---

## 3. Crate 详解

### 3.1 `protocol` — 契约层

- **路径**：`protocol/src/lib.rs`
- **职责**：Web / Desktop / Server / Shared 共用的序列化类型，避免各端手写 JSON 字段不一致。
- **典型类型**：
  - 聊天：`ChatTurnRequest`、`ConversationSummaryDto`、`StoredChatMessageDto`、`SseEvent`
  - Agent：`AgentRunDetailDto`、工具步骤/调用详情
  - 技能 / 角色 / LLM 配置 / 策略
  - 应用市场：`PluginSmartUiDto`（智能 UI 动作面板）等

依赖方向：**几乎无业务依赖**，被上层广泛引用。

### 3.2 `shared` — 运行时核心

- **路径**：`pusa-core/src/`
- **职责**：SQLite + Agent 循环 + 工具执行 + 策略调度；同时服务 **server 进程** 与 **桌面进程内 Runtime**。

```text
pusa-core/src/
├── lib.rs                 # RuntimeContext、对外 API
├── schema.sql             # 数据库 schema
├── strategy_scheduler.rs  # 策略定时调度
├── agent/                 # Agent 工具循环
├── core/
│   ├── db.rs              # SQLite 访问
│   ├── chat.rs            # 会话与消息
│   ├── llm.rs             # OpenAI 兼容调用
│   ├── skills.rs          # 技能安装/装备/市场
│   ├── credentials.rs     # 凭证
│   ├── system.rs          # 系统信息（如公网 IP）
│   ├── assets.rs          # 资产相关
│   └── strategy_semantic.rs
└── tools/
    ├── mod.rs             # 工具注册表
    ├── file.rs            # 读/写/编辑文件
    ├── skill.rs           # 技能相关工具
    └── exec_hint.rs
```

要点：

- `RuntimeContext` 持有 DB 连接、可选 `OPENAI_API_KEY`、策略事件广播通道。
- Desktop 通过 `ui/desktop/agent` 在本机打开同一套 runtime；Web 则经 `server` HTTP 访问。

### 3.3 `ui` — 界面层

- **路径**：`ui/src/`
- **职责**：唯一 UI crate；用 **feature** 区分 Web / Native。

```text
ui/src/
├── lib.rs                 # App 根组件、静态资源挂载
├── icons.rs               # 自定义 SVG 图标
├── shell/                 # 主控制台骨架
│   ├── console.rs         # 三栏布局、会话历史、标题栏开关等
│   ├── files.rs           # FILES 树、文件编辑器、行内 diff
│   ├── syntax.rs          # 代码语法高亮
│   ├── plugins.rs         # 我的应用 / 应用市场 / 智能 UI
│   ├── status_bar.rs      # 底栏
│   └── toast.rs           # 全局 toast
├── chat/                  # 聊天传输、Markdown、思考轨迹、附件
├── persona/               # 人设相关 UI
├── web/                   # 仅 WASM：鉴权、设置、LLM 配置、prefs
├── desktop/               # 仅 native：本机 agent、文件、终端、插件、窗口
└── components/            # 少量通用组件（主要 web）
```

**布局概念**（`shell::Console`）：

```text
┌──────── Sidebar ────────┬──── Center ────┬──── Chat ────┐
│ 导航：首页/文件/技能/应用 │  文件编辑/Diff  │  对话 + 历史  │
│ FILES / Skills / Apps   │  语法高亮+行号  │  右侧可拖宽   │
└─────────────────────────┴────────────────┴──── Terminal ┘
                                              （中间栏底部）
```

Feature 开关（见 `ui/Cargo.toml`）：

| Feature | 用途 |
|---------|------|
| `web` | WASM、浏览器 API、部分组件 |
| `native` | 桌面：shared 运行时、rfd、pty、objc/windows 等 |

### 3.4 `web` / `desktop` / `server` — 入口

| Crate | 入口 | 行为 |
|-------|------|------|
| `web` | `web/src/main.rs` | `dioxus::launch(ui::App)` |
| `desktop` | `desktop/src/main.rs` | 配置窗口 / Dock 图标 / boot HTML 后 launch |
| `server` | `server/src/main.rs` | Axum 路由 + `RuntimeContext` + 策略调度线程 |

桌面资源：`desktop/assets/`（图标、`boot-index.html`）。

---

## 4. 数据与内容流

### 4.1 聊天

```text
UI (chat facade)
  ├─ Web  → HTTP/SSE → server → shared::agent
  └─ Desktop → desktop::agent → shared::RuntimeContext（同进程）
       │
       ▼
  SQLite (data/server.db 或桌面本地库)
```

### 4.2 文件与工作区（Desktop）

- 侧栏 `FILES`：浏览工作区、自动轮询刷新、打开中间栏编辑器。
- 编辑器：行号 + 语法高亮叠层（`shell/syntax.rs`）。
- 聊天轨迹中「编辑/读取」可点：左侧选中文件、中间打开；编辑项展示行内 diff。

### 4.3 应用市场与智能 UI（Desktop）

- 安装目标：`applications/{id}/`
- 元数据：`.pusa-plugin.json`
- 智能 UI 缓存：`.pusa-smart-ui.json`（打开「我的应用」或安装后自动扫描生成）

逻辑主要在：

- `ui/src/desktop/plugins.rs` — 扫描、克隆、生成/读写智能 UI
- `ui/src/shell/plugins.rs` — 侧栏列表与详情 UI

### 4.4 技能

- 内容：`skills/*/SKILL.md`
- 运行时：`pusa-core/src/core/skills.rs` + `pusa-core/src/tools/skill.rs`
- UI：侧栏技能市场 / 已安装列表（`shell` + console 内嵌）

---

## 5. 构建与脚本

`justfile` 常用命令：

| 命令 | 作用 |
|------|------|
| `just web` | 启动 Web（`scripts/web.sh`） |
| `just server` | 启动 API（`scripts/server.sh`） |
| `just desktop` | 桌面热重载（`scripts/desktop.sh`） |
| `just desktop-mac` | 本机 macOS 打包 + 签名公证（`scripts/desktop-mac.sh`） |
| `just desktop-windows` | 本机 Windows 打包（默认不签名；`SIGN=1` → Azure Artifact Signing） |
| `just release-mac` | macOS 发版（tag 取自 `desktop/Cargo.toml`；可选 `BUILD=1`） |
| `just release-windows` | Windows 发版（同上；可选 `BUILD=1`） |
| `just db-clear` | 清空 `data/` |
| `just push` | 仅推送远端 |

环境变量示例：

- `DATABASE_PATH` — server SQLite 路径（默认 `./data/server.db`）
- `OPENAI_API_KEY` — LLM
- `ANOTHERME_BASE_URL` — 联调外部认证基址

桌面打包签名凭证见 `.env.signing.example`（复制为 `.env.signing`，已 gitignore）。

CI：`.github/workflows/`（桌面打包已改为本机流程，不再走 Actions）。

---

## 6. 改动定位速查

| 想改… | 优先看 |
|-------|--------|
| API 字段 / SSE 事件 | `protocol/src/lib.rs` |
| Agent 行为 / 工具 | `pusa-core/src/agent`、`pusa-core/src/tools` |
| HTTP 路由 | `server/src/main.rs` |
| 三栏布局 / 会话历史 | `ui/src/shell/console.rs` |
| 文件树 / 编辑器 / diff | `ui/src/shell/files.rs`、`syntax.rs` |
| 我的应用 / 智能 UI | `ui/src/shell/plugins.rs`、`ui/src/desktop/plugins.rs` |
| 聊天 Markdown / 思考轨迹 | `ui/src/chat/` |
| 桌面窗口 / 终端 / 本机 FS | `ui/src/desktop/`、`desktop/src/main.rs` |
| Web 设置 / 鉴权 | `ui/src/web/` |
| 样式 | `ui/assets/css/main.css`、`colors.css` |
| 技能包内容 | `skills/` |
| 已安装外部应用 | `applications/` |

---

## 7. 设计约定（摘要）

1. **协议先行**：跨端结构放 `protocol`，不要在 UI/server 各写一份。
2. **业务在 shared**：Agent、DB、工具只在一处实现；server 与 desktop 复用。
3. **UI 一分为三适配**：`shell` 共用布局；`web` / `desktop` 放平台能力。
4. **内容目录不进 workspace**：`skills` / `extension` / `application` 是运行时内容，不是 Cargo member。
5. **Dioxus 0.7**：用最新 hooks / 组件模型；详见根目录 `AGENTS.md`。

---

## 8. 相关文档

| 文件 | 内容 |
|------|------|
| `README.md` | 本地启动、桌面发版、签名说明 |
| `AGENTS.md` | Dioxus 0.7 API 速查 |
| `applications/README.md` | 外部应用目录约定 |
| `extensions/README.md` | 内部插件目录约定 |

---

*文档随仓库结构演进请同步更新。*
