//! 跨组件复用的 Tailwind 类片段。
//!
//! 为什么放 Rust 常量而不是 CSS 类：
//! - 样式仍是纯 Tailwind 工具类，`@source` 扫描 `ui/src/**/*.rs` 能直接提取；
//! - 避免「设置卡片 / 标题行 / 关闭钮」这套结构在多个调用点各写一遍长串，
//!   改一处即可全局生效。
//!
//! 注意：这里只能写**完整静态字面量**，不要用 `format!` 拼类名（Tailwind 扫不到）。
//!
//! 部分片段只在 web 构建的组件里使用（native 不编译那些模块），因此整模块允许
//! 未使用告警，避免两套 feature 下反复报 dead_code。
#![allow(dead_code)]

/// 弹窗卡片（窄版，28rem）：字体大小 / 关于本机。
pub(crate) const MODAL_CARD: &str = "relative w-full max-w-[28rem] h-auto max-h-full px-6 py-7 rounded-2xl border border-primary-2 bg-primary shadow-[0_16px_40px_var(--shadow-color)] flex flex-col min-h-0 overflow-hidden box-border dark:border-transparent dark:bg-[image:linear-gradient(var(--primary-color),var(--primary-color)),var(--ac-sand-grad-border)] dark:bg-origin-border dark:bg-clip-[padding-box,border-box] dark:shadow-[0_12px_36px_var(--shadow-color),inset_0_1px_0_rgb(255_255_255/0.05)]";

/// 弹窗卡片（宽版，32rem）：API 密钥设置页。
pub(crate) const MODAL_CARD_WIDE: &str = "relative w-full max-w-[32rem] h-auto max-h-full px-6 py-7 rounded-2xl border border-primary-2 bg-primary shadow-[0_16px_40px_var(--shadow-color)] flex flex-col min-h-0 overflow-hidden box-border dark:border-transparent dark:bg-[image:linear-gradient(var(--primary-color),var(--primary-color)),var(--ac-sand-grad-border)] dark:bg-origin-border dark:bg-clip-[padding-box,border-box] dark:shadow-[0_12px_36px_var(--shadow-color),inset_0_1px_0_rgb(255_255_255/0.05)]";

/// 卡片标题行：左标题 + 右侧绝对定位关闭钮，右侧留出 2.4rem 避免标题压到关闭钮。
pub(crate) const MODAL_HEAD: &str = "flex items-center justify-between gap-3 pr-[2.4rem] mb-4 shrink-0";

/// 卡片标题。
pub(crate) const MODAL_TITLE: &str = "m-0 text-[1.25rem] font-extrabold text-secondary-1";

/// 卡片右上角圆形关闭钮（内部图标加 `shrink-0` 即可）。
pub(crate) const MODAL_CLOSE: &str = "absolute top-4 right-4 inline-flex items-center justify-center w-8 h-8 p-0 border-0 rounded-full bg-primary-1 text-secondary-3 cursor-pointer hover:text-ac-blue hover:bg-border-light";

/// 卡片正文容器（纵向 flex），用于凭证面板。
pub(crate) const MODAL_BODY_FLEX_COL: &str =
    "flex flex-col flex-[0_1_auto] min-h-0 overflow-x-hidden overflow-y-auto";

/// 卡片正文容器（横向 flex，字号三选一）。
pub(crate) const MODAL_BODY_FLEX_ROW: &str =
    "flex gap-3 py-4 flex-[0_1_auto] min-h-0 overflow-x-hidden overflow-y-auto";

// ——————————————————————————— 设置表单字段 ———————————————————————————

/// 表单字段（label 包 input 的竖排结构）。
pub(crate) const FIELD_LABEL: &str =
    "flex flex-col gap-[0.35rem] m-0 mb-[0.85rem] text-[0.72rem] font-bold text-secondary-3";

/// 同上，但用于凭证卡的编辑区（行距更紧）。
pub(crate) const FIELD_LABEL_TIGHT: &str =
    "flex flex-col gap-[0.35rem] m-0 mb-[0.55rem] text-[0.72rem] font-bold text-secondary-3";

/// 文本 / 密码输入框；`disabled` 态用虚线描边区分「自动模式只读」。
pub(crate) const FIELD_INPUT: &str = "box-border w-full px-[0.65rem] py-[0.55rem] rounded-[0.5rem] border border-primary-2 bg-primary text-[0.8125rem] text-secondary-1 outline-none focus:border-border-dark focus:shadow-[0_0_0_3px_var(--shadow-color)] disabled:bg-primary-1 disabled:text-secondary-3 disabled:cursor-not-allowed disabled:border-dashed";

/// 密码输入框（右侧留出眼睛按钮的位置）。
pub(crate) const FIELD_INPUT_PASSWORD: &str = "box-border w-full pl-[0.65rem] pr-[2.75rem] py-[0.55rem] rounded-[0.5rem] border border-primary-2 bg-primary text-[0.8125rem] text-secondary-1 outline-none focus:border-border-dark focus:shadow-[0_0_0_3px_var(--shadow-color)] disabled:bg-primary-1 disabled:text-secondary-3 disabled:cursor-not-allowed disabled:border-dashed";

/// Provider 切换条容器（OpenAI / Anthropic）。
pub(crate) const PROVIDER_TABS: &str =
    "flex gap-1 m-0 mb-4 p-[0.2rem] rounded-[0.55rem] border border-border-light bg-primary-1";

/// 同上，凭证卡编辑区用。
pub(crate) const PROVIDER_TABS_TIGHT: &str =
    "flex gap-1 m-0 mb-[0.55rem] p-[0.2rem] rounded-[0.55rem] border border-border-light bg-primary-1";

/// Provider 切换按钮：状态交给 `data-active`，避免拼 `is-active`。
/// `data-[active=true]:hover:*` 是为了压过 hover 态（同一元素上两种状态同时成立时以激活态为准）。
pub(crate) const PROVIDER_TAB: &str = "flex-1 m-0 px-[0.65rem] py-[0.45rem] border-0 rounded-[0.4rem] bg-transparent text-secondary-3 text-[0.8125rem] font-[650] cursor-pointer transition-colors duration-150 hover:text-secondary-1 hover:bg-[color-mix(in_srgb,var(--primary-color-2)_55%,transparent)] data-[active=true]:bg-primary data-[active=true]:text-ac-blue data-[active=true]:shadow-[0_1px_4px_var(--shadow-color)] data-[active=true]:hover:bg-primary data-[active=true]:hover:text-ac-blue";

/// 表单底部双按钮行（取消 / 提交）。
pub(crate) const FIELD_ACTIONS_ROW: &str = "grid grid-cols-2 gap-[0.7rem] mt-5";

/// 同上，凭证卡编辑区用。
pub(crate) const FIELD_ACTIONS_ROW_TIGHT: &str = "grid grid-cols-2 gap-[0.7rem] mt-[0.15rem]";

/// 次级按钮（取消）。
pub(crate) const BTN_CANCEL: &str = "box-border w-full px-4 py-[0.65rem] border-0 rounded-[0.55rem] text-[0.9rem] font-extrabold text-secondary-1 bg-primary-1 cursor-pointer hover:bg-[color-mix(in_srgb,var(--primary-color-2)_70%,var(--primary-color-1))]";

/// 主按钮（提交 / 保存）。
pub(crate) const BTN_SUBMIT: &str = "box-border w-full px-4 py-[0.65rem] border-0 rounded-[0.55rem] text-[0.9rem] font-extrabold text-primary bg-button-primary cursor-pointer disabled:opacity-55 disabled:cursor-not-allowed";

/// 胶囊「添加」按钮：与聊天栏发送钮同色同高。
pub(crate) const BTN_ADD_PILL: &str = "box-border w-full mt-[0.15rem] h-9 min-h-9 px-4 border-0 rounded-full inline-flex items-center justify-center text-[0.9rem] font-[650] text-primary bg-[var(--ac-chat-send-ready)] shadow-[0_14px_30px_var(--shadow-color)] cursor-pointer transition-all duration-150 hover:bg-[var(--ac-chat-send-ready-hover)] hover:text-white hover:-translate-y-px focus-visible:bg-[var(--ac-chat-send-ready-hover)] focus-visible:text-white disabled:opacity-45 disabled:cursor-not-allowed";

// ——————————————————————————— LLM 凭证卡 ———————————————————————————

/// 凭证卡列表容器。
pub(crate) const CRED_LIST: &str =
    "list-none m-0 mb-3 p-0 flex flex-col gap-[0.55rem] flex-[0_1_auto]";

/// 列表底部「添加密钥」区（不随列表滚动）。
pub(crate) const CRED_FOOT: &str = "flex-[0_0_auto]";

/// 单张凭证卡：`data-on=true` 时描边/外发光变蓝。
pub(crate) const CRED_CARD: &str = "flex flex-col items-stretch gap-0 px-[0.8rem] py-[0.7rem] rounded-[0.85rem] border border-primary-2 bg-[image:linear-gradient(180deg,var(--primary-color)_0%,var(--primary-color-3)_100%)] shadow-[0_10px_24px_var(--shadow-color)] data-[on=true]:border-[color-mix(in_srgb,var(--ac-color-2563eb)_45%,var(--primary-color-2))] data-[on=true]:shadow-[0_10px_24px_var(--shadow-color),0_0_0_1px_color-mix(in_srgb,var(--ac-color-2563eb)_22%,transparent)]";

/// 卡片头行：左侧信息 + 右侧操作。
pub(crate) const CRED_ROW: &str = "flex items-center justify-between gap-3 w-full";

/// 卡片内编辑区（上方一条分割线）。
pub(crate) const CRED_EDIT: &str = "mt-[0.7rem] pt-[0.7rem] border-t border-primary-2";

/// 卡片左侧信息列。
pub(crate) const CRED_MAIN: &str = "min-w-0 flex-[1_1_auto]";

/// 名称行。
pub(crate) const CRED_TOP: &str = "flex items-center gap-[0.45rem] mb-[0.2rem]";

/// 名称文本。
pub(crate) const CRED_NAME: &str = "text-[0.78rem] font-[750] text-secondary-1";

/// 掩码后的 key。
pub(crate) const CRED_KEY: &str = "text-[0.72rem] leading-[1.4] overflow-hidden text-ellipsis whitespace-nowrap font-mono text-secondary-1";

/// base url。
pub(crate) const CRED_BASE: &str = "text-[0.72rem] leading-[1.4] overflow-hidden text-ellipsis whitespace-nowrap text-secondary-3";

/// 右侧操作组（编辑 / 删除 / 开关）。
pub(crate) const CRED_ACTIONS: &str = "flex flex-row items-center gap-[0.4rem] shrink-0";

/// 图标按钮（编辑）。
pub(crate) const CRED_ICON_BTN: &str = "m-0 w-7 h-7 p-0 border-0 rounded-[0.4rem] inline-flex items-center justify-center bg-transparent text-secondary-3 cursor-pointer hover:text-ac-blue hover:bg-[color-mix(in_srgb,var(--ac-color-2563eb)_12%,transparent)]";

/// 图标按钮（删除）。注意：原 CSS 的红色 hover 依赖 colors.css 里不存在的
/// `--ac-color-dc2626`，实际渲染是「无底色 + 文字回归正文色」，这里按现状复刻；
/// 若要真的变红，需先在 colors.css 补该令牌。
pub(crate) const CRED_ICON_BTN_DANGER: &str = "m-0 w-7 h-7 p-0 border-0 rounded-[0.4rem] inline-flex items-center justify-center bg-transparent text-secondary-3 cursor-pointer hover:text-text-primary hover:bg-transparent";

// ——————————————————————————— 扩展 / 技能库 ———————————————————————————

/// 列表行的图标位（1.85rem 方块 + 蓝色系底）。
pub(crate) const SKILL_GLYPH_LIST_BLUE: &str = "inline-flex items-center justify-center shrink-0 w-[1.85rem] h-[1.85rem] rounded-[0.4rem] mt-[0.05rem] text-[var(--ac-color-0891b2)] bg-[var(--ac-color-ecfeff)]";

/// 同上，有远程图标时（底色透明 + 裁切）。
pub(crate) const SKILL_GLYPH_LIST_IMG: &str = "inline-flex items-center justify-center shrink-0 w-[1.85rem] h-[1.85rem] rounded-[0.4rem] mt-[0.05rem] bg-transparent overflow-hidden";

/// 详情页大图标位（3rem 方块）。
pub(crate) const SKILL_GLYPH_DETAIL_BLUE: &str = "inline-flex items-center justify-center shrink-0 w-[3rem] h-[3rem] rounded-[0.75rem] text-[var(--ac-color-0891b2)] bg-[var(--ac-color-ecfeff)]";

/// 同上，有远程图标时。
pub(crate) const SKILL_GLYPH_DETAIL_IMG: &str =
    "inline-flex items-center justify-center shrink-0 w-[3rem] h-[3rem] rounded-[0.75rem] bg-transparent overflow-hidden";

/// 图标位里的 `<img>`。
pub(crate) const SKILL_GLYPH_IMG: &str = "w-full h-full object-cover rounded-[inherit] block";

/// 无图标时的首字母。
pub(crate) const SKILL_GLYPH_LETTER: &str = "font-[800] text-[0.78rem] leading-none select-none";

/// 详情页大图标位里的首字母。
pub(crate) const SKILL_GLYPH_LETTER_LARGE: &str =
    "font-[800] text-[1.15rem] leading-none select-none";

/// 侧栏扩展面板容器。
pub(crate) const SKILLS_PANEL: &str =
    "flex flex-col gap-[0.65rem] min-h-0 flex-1 -mt-[0.35rem] -mx-[0.15rem] mb-0";

/// 面板顶部搜索胶囊。
pub(crate) const SKILLS_SEARCH: &str = "flex items-center gap-[0.4rem] h-8 pl-[0.65rem] pr-3 rounded-full border border-border-light bg-[color-mix(in_srgb,var(--primary-color)_88%,var(--primary-color-1))] box-border shrink-0";

/// 搜索框本体（去掉原生边框，占满剩余宽度）。
pub(crate) const SKILLS_SEARCH_INPUT: &str = "min-w-0 flex-1 border-0 outline-0 bg-transparent text-secondary-1 text-[0.78rem] font-[650] placeholder:text-secondary-3 placeholder:opacity-85";

/// 「我的 / 市场」分段切换容器。
pub(crate) const SKILLS_TOGGLE: &str = "flex items-center gap-1 p-[0.15rem] rounded-[0.45rem] bg-[color-mix(in_srgb,var(--border-color-light)_70%,transparent)] shrink-0";

/// 分段切换按钮（状态交给 `data-active`）。
pub(crate) const SKILLS_TOGGLE_BTN: &str = "flex-1 min-w-0 h-[1.7rem] px-[0.35rem] border-0 rounded-[0.35rem] inline-flex items-center justify-center gap-[0.28rem] bg-transparent text-secondary-3 text-[0.72rem] font-extrabold whitespace-nowrap cursor-pointer data-[active=true]:text-secondary-1 data-[active=true]:bg-primary data-[active=true]:shadow-[0_1px_3px_var(--shadow-color)] data-[active=true]:hover:text-secondary-1 data-[active=true]:hover:bg-primary";

/// 提示条（Web 端不可安装）。
pub(crate) const SKILLS_BANNER_WARN: &str = "px-[0.55rem] py-[0.45rem] rounded-[0.45rem] border border-[var(--ac-color-fde68a)] bg-[var(--ac-color-fffbeb)] text-[var(--ac-color-92400e)] text-[0.68rem] font-bold leading-[1.45] flex flex-col gap-[0.2rem] shrink-0";

/// 错误条。
pub(crate) const SKILLS_BANNER_ERROR: &str = "px-[0.55rem] py-[0.45rem] rounded-[0.45rem] border border-[var(--ac-color-fecaca)] bg-[color-mix(in_srgb,var(--ac-color-fecaca)_55%,var(--primary-color))] text-[var(--ac-color-991b1b)] text-[0.68rem] font-bold leading-[1.45] flex flex-col gap-[0.2rem] shrink-0";

/// 空列表提示。
pub(crate) const SKILLS_EMPTY: &str = "px-[0.6rem] py-[1.1rem] text-center text-secondary-3 text-[0.74rem] font-[650] leading-[1.5] shrink-0";

/// 列表分组标题。
pub(crate) const SKILLS_SECTION_HEADER: &str = "flex items-center justify-between gap-2 pt-[0.15rem] px-[0.35rem] pb-0 text-secondary-3 text-[0.68rem] font-extrabold tracking-[0.06em] uppercase shrink-0";

/// 列表滚动容器。
pub(crate) const SKILLS_LIST: &str = "flex flex-col gap-[0.1rem] pb-2 flex-1 min-h-0 overflow-y-auto";

/// 列表行（状态交给 `data-selected`）。
pub(crate) const SKILLS_ROW: &str = "flex items-start gap-[0.55rem] px-[0.4rem] py-[0.45rem] rounded-[0.4rem] border border-transparent cursor-pointer transition-[background-color,border-color,box-shadow] duration-100 hover:bg-[color-mix(in_srgb,var(--border-color-light)_80%,transparent)] data-[selected=true]:bg-[var(--skills-list-selected-bg)] data-[selected=true]:hover:bg-[var(--skills-list-selected-bg)]";

/// 骨架行（禁止交互）。
pub(crate) const SKILLS_ROW_SKELETON: &str = "pointer-events-none";

/// 列表行右侧文字列。
pub(crate) const SKILLS_LIST_BODY: &str = "min-w-0 flex-1 flex flex-col gap-[0.12rem]";

/// 列表行标题。
pub(crate) const SKILLS_LIST_TITLE: &str =
    "text-secondary-1 text-[0.8rem] font-extrabold leading-[1.25] truncate";

/// 列表行描述（两行截断）。
pub(crate) const SKILLS_LIST_DESC: &str =
    "text-secondary-3 text-[0.7rem] font-semibold leading-[1.35] line-clamp-2";

/// 列表行元信息。
pub(crate) const SKILLS_LIST_META: &str = "text-[color-mix(in_srgb,var(--secondary-color-3)_88%,var(--primary-color-5))] text-[0.62rem] font-[650] truncate";

/// 骨架屏标题条。
pub(crate) const SKILLS_SKEL_TITLE: &str = "w-[62%] h-[0.72rem] rounded-[0.25rem]";

/// 骨架屏描述条。
pub(crate) const SKILLS_SKEL_DESC: &str = "w-[88%] h-[0.62rem] mt-[0.25rem] rounded-[0.25rem]";

/// 详情页根容器。
pub(crate) const SKILL_DETAIL_PANE: &str = "w-full max-w-none m-0 p-0 min-h-full box-border";

/// 详情内容列。
pub(crate) const SKILL_DETAIL: &str =
    "flex flex-col gap-4 px-[1.1rem] pt-[1.1rem] pb-[1.4rem] border-0 rounded-none bg-transparent";

/// 详情头部（大图标 + 标题）。
pub(crate) const SKILL_DETAIL_HEAD: &str = "flex items-start gap-[0.9rem]";

/// 详情标题列。
pub(crate) const SKILL_DETAIL_TITLES: &str = "min-w-0 flex flex-col gap-1";

/// 详情标题（h2）。
pub(crate) const SKILL_DETAIL_TITLE: &str =
    "m-0 text-secondary-1 text-[1.2rem] font-[850] tracking-[-0.02em] leading-[1.25]";

/// 详情路径行。
pub(crate) const SKILL_DETAIL_SLUG: &str =
    "m-0 text-secondary-3 text-[0.78rem] font-[650] leading-[1.35] [overflow-wrap:anywhere]";

/// 详情描述。
pub(crate) const SKILL_DETAIL_DESC: &str =
    "m-0 text-secondary-3 text-[0.9rem] leading-[1.65] font-[650]";

/// 详情底部操作区。
pub(crate) const SKILL_DETAIL_ACTIONS: &str =
    "flex flex-wrap items-center gap-[0.55rem] pt-[0.35rem]";

/// 详情空态容器。
pub(crate) const SKILL_DETAIL_EMPTY: &str = "flex flex-col items-center justify-center text-center gap-[0.65rem] min-h-[16rem] px-5 py-8 text-secondary-3";

/// 详情空态标题。
pub(crate) const SKILL_DETAIL_EMPTY_H3: &str = "m-0 text-[1.05rem] font-extrabold text-secondary-3";

/// 详情空态说明。
pub(crate) const SKILL_DETAIL_EMPTY_P: &str = "m-0 max-w-[22rem] text-[0.86rem] leading-[1.6] font-[650]";

/// 空态里的大图标（半透明）。
pub(crate) const SKILL_EMPTY_ICON: &str = "opacity-[0.45] text-[var(--primary-color-5)]";

/// 智能 UI 面板。
pub(crate) const SMART_UI_PANEL: &str = "mt-3 mx-0 mb-[0.35rem] px-[0.95rem] py-[0.85rem] rounded-[1rem] border border-primary-2 bg-[linear-gradient(180deg,color-mix(in_srgb,var(--primary-color-3)_55%,transparent)_0%,var(--primary-color)_100%)]";

/// 智能 UI 头部行。
pub(crate) const SMART_UI_HEAD: &str = "flex items-start justify-between gap-3 mb-[0.7rem]";

/// 智能 UI 标题。
pub(crate) const SMART_UI_TITLE: &str = "m-0 text-[0.95rem] font-extrabold text-secondary-1";

/// 智能 UI 摘要。
pub(crate) const SMART_UI_SUMMARY: &str =
    "mt-[0.35rem] mx-0 mb-0 text-[0.8rem] leading-[1.5] text-secondary-3 font-semibold";

/// 「重新生成」按钮。
pub(crate) const SMART_UI_REGEN: &str = "h-8 px-[0.7rem] rounded-[0.65rem] border border-primary-2 bg-primary text-secondary-1 inline-flex items-center gap-[0.35rem] text-[0.72rem] font-bold cursor-pointer shrink-0 enabled:hover:bg-primary-3 disabled:opacity-55 disabled:cursor-wait";

/// 动作按钮组。
pub(crate) const SMART_UI_ACTIONS: &str = "flex flex-wrap gap-2";

/// 单个动作按钮。
pub(crate) const SMART_UI_ACTION: &str = "h-[2.2rem] px-[0.8rem] rounded-[0.7rem] border border-[var(--ac-chat-toolbar-pill-border)] bg-[var(--ac-chat-toolbar-pill-bg)] text-secondary-1 inline-flex items-center gap-[0.35rem] text-[0.78rem] font-extrabold cursor-pointer shadow-[var(--ac-chat-toolbar-pill-shadow)] transition-[border-color,box-shadow,background-color,transform] duration-150 enabled:hover:border-[color-mix(in_srgb,var(--ac-chat-toolbar-pill-border)_55%,var(--border-color-hover))] enabled:hover:shadow-[0_3px_8px_#0f172a14] enabled:hover:-translate-y-px enabled:active:scale-[0.98] disabled:opacity-55 disabled:cursor-wait disabled:translate-y-0";

/// 智能 UI 加载文案。
pub(crate) const SMART_UI_LOADING_TEXT: &str = "m-0 text-[0.85rem] text-secondary-3 font-[650]";

/// 「学习 / 打开目录 / 复制路径 / 智能UI」实心次级按钮（浅色渐变 + 阴影）。
pub(crate) const BTN_SKILL_EQUIP: &str = "h-[2.4rem] min-w-[6rem] w-auto max-w-none flex-[0_0_auto] px-[0.9rem] rounded-[0.8rem] border border-primary-2 bg-[linear-gradient(180deg,var(--primary-color)_0%,var(--primary-color-3)_100%)] text-secondary-1 inline-flex items-center justify-center gap-[0.4rem] text-[0.8rem] font-extrabold tracking-[0.01em] cursor-pointer box-border shadow-[0_10px_24px_var(--shadow-color)] transition-[background-color,color,box-shadow,transform] duration-150 enabled:hover:shadow-[0_16px_32px_var(--shadow-color)] enabled:hover:-translate-y-px enabled:active:translate-y-0 enabled:active:shadow-[0_8px_18px_var(--shadow-color)] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:color-mix(in_srgb,var(--brand)_40%,transparent)] disabled:opacity-55 disabled:cursor-wait disabled:shadow-none disabled:translate-y-0 [&_svg]:opacity-90";

/// 「卸载」浅红次级按钮。
pub(crate) const BTN_SKILL_REMOVE: &str = "h-[2.4rem] min-w-[6rem] w-auto max-w-none flex-[0_0_auto] px-[0.9rem] rounded-[0.8rem] border border-transparent text-[var(--ac-color-991b1b)] bg-[color-mix(in_srgb,var(--ac-color-991b1b)_8%,var(--primary-color))] inline-flex items-center justify-center gap-[0.4rem] text-[0.8rem] font-extrabold cursor-pointer box-border transition-[background-color,color,box-shadow,transform] duration-150 enabled:hover:bg-[color-mix(in_srgb,var(--ac-color-991b1b)_14%,var(--primary-color))] enabled:hover:-translate-y-px enabled:active:bg-[color-mix(in_srgb,var(--ac-color-991b1b)_22%,var(--primary-color))] enabled:active:translate-y-0 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:color-mix(in_srgb,var(--ac-color-991b1b)_38%,var(--brand))] disabled:cursor-not-allowed disabled:opacity-55 disabled:translate-y-0 [&_svg]:opacity-92";

/// 「安装」主按钮（`a` 标签也用它）。
pub(crate) const BTN_SKILL_INSTALL: &str = "h-[2.4rem] min-w-[6rem] px-[0.9rem] rounded-[0.8rem] border border-transparent inline-flex items-center justify-center gap-[0.4rem] text-[0.8rem] font-extrabold text-primary bg-[var(--button-primary)] shadow-[0_12px_28px_var(--shadow-color)] cursor-pointer no-underline";

/// 不可安装时的弱化按钮。
pub(crate) const BTN_SKILL_INSTALLED: &str = "h-[2.4rem] min-w-[6rem] px-[0.9rem] rounded-[0.8rem] border border-transparent inline-flex items-center justify-center gap-[0.4rem] text-[0.8rem] font-extrabold text-secondary-3 bg-[var(--ac-color-eff6ff)] cursor-default";

// ——————————————————————————— 侧栏内容搜索 ———————————————————————————

/// 搜索面板容器。
pub(crate) const SEARCH_PANEL: &str =
    "flex flex-col gap-[0.4rem] min-h-0 flex-1 -mt-[0.35rem] -mx-[0.35rem] mb-0";

/// 面板标题行（`group` 用于让右侧操作区在悬停时变实）。
pub(crate) const SEARCH_HEADER: &str =
    "group flex items-center gap-1 min-h-[1.65rem] pl-[0.35rem] pr-[0.15rem] py-0";

/// 标题文字。
pub(crate) const SEARCH_TITLE: &str =
    "min-w-0 flex-1 text-[0.72rem] font-extrabold tracking-[0.08em] uppercase text-secondary-1 truncate";

/// 标题行右侧操作组。
pub(crate) const SEARCH_ACTIONS: &str =
    "inline-flex items-center gap-[0.05rem] shrink-0 opacity-[0.72] group-hover:opacity-100";

/// 字段区（查询行 / 替换行 / 过滤条件）。
pub(crate) const SEARCH_FIELDS: &str = "flex flex-col gap-[0.4rem] px-[0.3rem] shrink-0";

/// 「展开/收起替换」成对结构。
pub(crate) const SEARCH_PAIR: &str = "flex items-start gap-[0.15rem]";

/// 折叠箭头按钮。
pub(crate) const SEARCH_PAIR_TOGGLE: &str = "w-[1.35rem] h-[1.7rem] p-0 border-0 rounded-[0.3rem] inline-flex items-center justify-center text-secondary-3 bg-transparent cursor-pointer shrink-0 hover:text-secondary-1 hover:bg-[color-mix(in_srgb,var(--primary-color-2)_70%,transparent)]";

/// 同上，过滤器行用（更矮，与 label 基线对齐）。
pub(crate) const SEARCH_PAIR_TOGGLE_SHORT: &str = "w-[1.35rem] h-[1.15rem] mt-[0.02rem] p-0 border-0 rounded-[0.3rem] inline-flex items-center justify-center text-secondary-3 bg-transparent cursor-pointer shrink-0 hover:text-secondary-1 hover:bg-[color-mix(in_srgb,var(--primary-color-2)_70%,transparent)]";

/// 输入行右侧内容列。
pub(crate) const SEARCH_PAIR_INPUTS: &str = "min-w-0 flex-1 flex flex-col gap-[0.28rem]";

/// 带描边的输入行（内含输入框 + 开关按钮）。
pub(crate) const SEARCH_INPUT_ROW: &str = "flex items-center gap-[0.12rem] min-h-[1.7rem] pl-2 pr-[0.2rem] rounded-[0.4rem] border border-border-light bg-[color-mix(in_srgb,var(--primary-color)_88%,var(--primary-color-1))] box-border focus-within:border-[color-mix(in_srgb,var(--brand)_55%,var(--border-color-light))]";

/// 输入框本体。
pub(crate) const SEARCH_INPUT: &str = "min-w-0 flex-1 h-[1.6rem] border-0 outline-0 bg-transparent text-secondary-1 text-[0.78rem] font-[650] placeholder:text-secondary-3 placeholder:opacity-85";

/// 查询选项开关（Aa / 全字 / 正则 / 保留大小写）；激活态由 `data-active` 驱动。
pub(crate) const SEARCH_TOGGLE: &str = "w-[1.35rem] h-[1.35rem] p-0 border-0 rounded-[0.28rem] inline-flex items-center justify-center text-secondary-3 bg-transparent cursor-pointer shrink-0 hover:text-secondary-1 hover:bg-[color-mix(in_srgb,var(--primary-color-2)_70%,transparent)] data-[active=true]:text-brand data-[active=true]:bg-[color-mix(in_srgb,var(--brand)_16%,transparent)] data-[active=true]:hover:text-brand data-[active=true]:hover:bg-[color-mix(in_srgb,var(--brand)_16%,transparent)] disabled:opacity-40 disabled:cursor-default";

/// 过滤器字段（label 包 label + input）。
pub(crate) const SEARCH_FILTER: &str = "flex flex-col gap-[0.18rem]";

/// 过滤器字段名。
pub(crate) const SEARCH_FILTER_LABEL: &str = "text-[0.66rem] font-bold text-secondary-3 tracking-[0.02em] flex items-center min-h-[1.15rem] px-[0.15rem]";

/// 过滤器输入框。
pub(crate) const SEARCH_FILTER_INPUT: &str = "w-full box-border h-[1.65rem] px-2 rounded-[0.35rem] border border-border-light bg-[color-mix(in_srgb,var(--primary-color)_88%,var(--primary-color-1))] text-secondary-1 text-[0.74rem] font-semibold outline-none focus:border-[color-mix(in_srgb,var(--brand)_55%,var(--border-color-light))] placeholder:text-secondary-3 placeholder:opacity-80";

/// 结果数量摘要。
pub(crate) const SEARCH_SUMMARY: &str = "px-[0.45rem] pt-[0.1rem] pb-[0.15rem] text-[0.68rem] font-[650] text-secondary-3 shrink-0";

/// 结果滚动区。
pub(crate) const SEARCH_RESULTS: &str = "flex flex-col gap-[0.08rem] min-h-0 flex-1 pt-[0.05rem] pr-0 pb-[0.4rem] pl-[0.1rem] overflow-y-auto";

/// 单个文件分组。
pub(crate) const SEARCH_FILE: &str = "flex flex-col min-w-0";

/// 文件分组的头行（可点开/折叠）。
pub(crate) const SEARCH_FILE_HEAD: &str = "flex items-center gap-[0.2rem] w-full min-h-[1.45rem] px-[0.35rem] py-[0.1rem] border-0 rounded-[0.3rem] bg-transparent text-secondary-1 cursor-pointer text-left box-border hover:bg-[color-mix(in_srgb,var(--primary-color-2)_58%,transparent)]";

/// 文件名。
pub(crate) const SEARCH_FILE_NAME: &str = "min-w-0 flex-1 text-[0.76rem] font-bold leading-[1.2] truncate";

/// 命中数量。
pub(crate) const SEARCH_FILE_COUNT: &str = "shrink-0 text-[0.64rem] font-bold text-secondary-3 min-w-[1.1rem] text-right";

/// 单条命中（缩进显示）。
pub(crate) const SEARCH_HIT: &str = "flex items-start gap-[0.35rem] w-full pl-[1.35rem] pr-[0.4rem] py-[0.12rem] border-0 rounded-[0.3rem] bg-transparent text-secondary-1 cursor-pointer text-left box-border hover:bg-[color-mix(in_srgb,var(--primary-color-2)_58%,transparent)]";

/// 命中行号。
pub(crate) const SEARCH_HIT_LN: &str = "shrink-0 min-w-[1.5rem] text-[0.66rem] font-[650] tabular-nums text-secondary-3 text-right pt-[0.08rem]";

/// 命中行文本（等宽、不换行）。
pub(crate) const SEARCH_HIT_TEXT: &str = "min-w-0 flex-1 text-[0.72rem] font-[550] leading-[1.35] whitespace-pre overflow-hidden text-ellipsis font-mono";

/// 命中片段高亮。
pub(crate) const SEARCH_HIT_MARK: &str = "bg-[color-mix(in_srgb,var(--brand)_32%,transparent)] text-secondary-1 rounded-[0.12rem] font-[750]";

// ——————————————————————————— 侧栏文件树 ———————————————————————————

/// 文件面板容器（与搜索面板同尺寸）。
pub(crate) const FILES_PANEL: &str =
    "flex flex-col gap-[0.4rem] min-h-0 flex-1 -mt-[0.35rem] -mx-[0.35rem] mb-0";

/// 面板标题行（`group` 让右侧操作区悬停时变实）。
pub(crate) const FILES_HEADER: &str =
    "group flex items-center gap-1 min-h-[1.65rem] pl-[0.05rem] pr-[0.15rem] py-0";

/// 标题按钮（点一下展开/收起整棵树）。
pub(crate) const FILES_TITLE_BTN: &str = "inline-flex items-center gap-[0.2rem] min-w-0 flex-1 px-[0.2rem] py-[0.15rem] border-0 rounded-[0.35rem] bg-transparent text-secondary-1 cursor-pointer text-left hover:bg-[color-mix(in_srgb,var(--primary-color-2)_55%,transparent)]";

/// 标题行里的展开箭头位。
pub(crate) const FILES_TITLE_CHEVRON: &str = "inline-flex text-secondary-3 shrink-0";

/// 工作区根目录名。
pub(crate) const FILES_ROOT_NAME: &str =
    "text-[0.72rem] font-extrabold tracking-[0.08em] uppercase truncate";

/// 标题行右侧操作组。
pub(crate) const FILES_ACTIONS: &str =
    "inline-flex items-center gap-[0.05rem] shrink-0 opacity-[0.72] group-hover:opacity-100";

/// 图标按钮（新建文件夹 / 新建文件 / 刷新 / 全部折叠）。
pub(crate) const FILES_ACTION: &str = "w-[1.35rem] h-[1.35rem] p-0 border-0 rounded-[0.3rem] inline-flex items-center justify-center text-secondary-3 bg-transparent cursor-pointer hover:text-secondary-1 hover:bg-[color-mix(in_srgb,var(--primary-color-2)_70%,transparent)]";

/// 新建文件/文件夹的浮层表单。
pub(crate) const FILES_CREATE: &str = "flex flex-col gap-[0.3rem] mx-[0.25rem] px-[0.45rem] py-[0.4rem] rounded-[0.4rem] border border-border-light bg-[color-mix(in_srgb,var(--primary-color)_90%,var(--primary-color-1))]";

/// 表单标题（新建文件 / 新建文件夹）。
pub(crate) const FILES_CREATE_LABEL: &str = "text-[0.68rem] font-bold text-secondary-3";

/// 表单里的目标目录。
pub(crate) const FILES_CREATE_PARENT: &str =
    "text-[0.66rem] font-semibold text-secondary-3 opacity-85 truncate";

/// 表单输入框。
pub(crate) const FILES_CREATE_INPUT: &str = "w-full box-border h-[1.7rem] px-[0.45rem] rounded-[0.3rem] border border-border-light bg-primary text-secondary-1 text-[0.78rem] font-semibold outline-none focus:border-[color-mix(in_srgb,var(--ac-color-2b7fff)_55%,var(--border-color-light))]";

/// 表单「取消」。
pub(crate) const FILES_CREATE_CANCEL: &str = "self-end border-0 bg-transparent text-secondary-3 text-[0.7rem] font-[650] cursor-pointer px-[0.2rem] py-[0.1rem] hover:text-secondary-1";

/// 树滚动容器。
pub(crate) const FILES_TREE: &str =
    "flex flex-col gap-[0.05rem] min-h-0 flex-1 pt-[0.1rem] pr-0 pb-[0.4rem] pl-[0.15rem] overflow-y-auto";

/// 树里的行按钮：选中 / 弱化 / 刚写入三种状态交给 `data-selected` / `data-muted` / `data-written`。
/// `group` 供行名在弱化态降字重。写入脉冲动画来自 app.css 的 `--animate-files-just-written`。
pub(crate) const FILES_ROW: &str = "group flex items-center gap-[0.2rem] w-full min-h-[1.45rem] px-[0.35rem] py-[0.1rem] border-0 rounded-[0.3rem] bg-transparent text-secondary-1 cursor-pointer text-left box-border hover:bg-[color-mix(in_srgb,var(--primary-color-2)_58%,transparent)] data-[selected=true]:bg-[color-mix(in_srgb,var(--ac-color-93c5fd)_28%,var(--primary-color-1))] data-[muted=true]:text-secondary-3 data-[muted=true]:opacity-[0.58] data-[written=true]:bg-[color-mix(in_srgb,var(--brand)_22%,transparent)] data-[written=true]:shadow-[inset_2px_0_0_var(--brand)] data-[written=true]:animate-files-just-written data-[selected=true]:data-[written=true]:bg-[color-mix(in_srgb,var(--ac-color-93c5fd)_42%,var(--primary-color-1))]";

/// 重命名中的行（输入框替代文件名）。
pub(crate) const FILES_ROW_RENAMING: &str = "flex items-center gap-[0.25rem] w-full min-h-[1.45rem] border-0 rounded-[0.3rem] bg-transparent text-secondary-1 cursor-pointer text-left box-border pt-[0.15rem] pb-[0.15rem] pr-[0.4rem]";

/// 行内展开箭头位。
pub(crate) const FILES_ROW_CHEVRON: &str = "w-3 h-3 inline-flex items-center justify-center shrink-0 text-secondary-3";

/// 行内文件/文件夹图标位。
pub(crate) const FILES_ROW_ICON: &str = "inline-flex shrink-0 text-secondary-3";

/// 行内文件名。
pub(crate) const FILES_ROW_NAME: &str = "min-w-0 flex-1 text-[0.78rem] font-semibold leading-[1.2] truncate group-data-[muted=true]:font-[550]";

/// 行内重命名输入框（`data-files-rename-input` 供 JS 聚焦钩子定位）。
pub(crate) const FILES_RENAME_INPUT: &str = "flex-1 min-w-0 m-0 px-[0.35rem] py-[0.15rem] border border-primary-2 rounded-[0.3rem] bg-primary text-secondary-3 text-[0.78rem] focus:outline-none focus:border-[var(--accent-color,var(--primary-color-3))]";

/// 树内空/错误提示。
pub(crate) const FILES_EMPTY: &str =
    "px-[0.55rem] py-3 text-[0.75rem] font-semibold text-secondary-3 text-center";

/// 同上，错误态（红字）。
pub(crate) const FILES_EMPTY_ERROR: &str =
    "px-[0.55rem] py-3 text-[0.75rem] font-semibold text-error text-center";

/// Web 端不可用的整块说明。
pub(crate) const FILES_UNAVAILABLE: &str =
    "px-[0.55rem] py-[0.85rem] text-center text-secondary-3 text-[0.78rem] font-[650] leading-[1.45]";

/// 同上，第二行小字。
pub(crate) const FILES_UNAVAILABLE_HINT: &str =
    "mt-[0.35rem] text-[0.72rem] font-[550] opacity-90";

// ——————————————————————————— 右键菜单（文件树 / 会话历史共用） ———————————————————————————

/// 点击穿透遮罩（`fixed` 铺满，透明）。
pub(crate) const CTX_BACKDROP: &str = "fixed inset-0 z-[1200] bg-transparent";

/// 菜单本体。暗色主题下换成「磨砂黑 + 渐变细边」，用 `dark:` 复刻原
/// `html[data-theme="dark"] .ac-chat-history-ctx-menu` 的 background-clip 技巧。
pub(crate) const CTX_MENU: &str = "fixed z-[1201] min-w-[8.75rem] p-[0.3rem] rounded-[0.55rem] border border-primary-2 bg-primary shadow-[0_2px_6px_var(--shadow-color),0_14px_36px_-6px_var(--shadow-color)] dark:border-transparent dark:[background-image:linear-gradient(var(--primary-color),var(--primary-color)),var(--ac-sand-grad-border)] dark:[background-origin:border-box] dark:[background-clip:padding-box,border-box] dark:shadow-[0_12px_36px_var(--shadow-color),inset_0_1px_0_rgb(255_255_255/0.05)]";

/// 菜单项：`:hover:not(:disabled)` 用 `enabled:hover:` 表达。
pub(crate) const CTX_ITEM: &str = "flex items-center gap-[0.45rem] w-full box-border m-0 px-[0.55rem] py-[0.45rem] border-0 rounded-[0.4rem] bg-transparent text-[0.8rem] font-[650] text-secondary-3 text-left cursor-pointer whitespace-nowrap enabled:hover:bg-primary-1 disabled:opacity-50 disabled:cursor-not-allowed";

/// 菜单项：危险操作（删除）。
pub(crate) const CTX_ITEM_DANGER: &str = "flex items-center gap-[0.45rem] w-full box-border m-0 px-[0.55rem] py-[0.45rem] border-0 rounded-[0.4rem] bg-transparent text-[0.8rem] font-[650] text-[var(--ac-color-ef4444,#ef4444)] text-left cursor-pointer whitespace-nowrap enabled:hover:bg-[color-mix(in_srgb,var(--ac-color-ef4444,#ef4444)_10%,transparent)] disabled:opacity-50 disabled:cursor-not-allowed";

/// 文件树菜单：更宽 + 可滚动 + 层级高于中间栏（`!` 压过 `CTX_MENU` 的 z-1201）。
pub(crate) const FS_CTX_MENU: &str =
    "min-w-[15.5rem] max-h-[min(70vh,32rem)] overflow-y-auto z-[1301]!";

/// 文件树菜单里的分割线。
pub(crate) const FS_CTX_SEP: &str = "h-px mx-[0.4rem] my-[0.28rem] bg-primary-2";

// ——————————————————————————— 中间栏预览器（md / html / 图片 / 音频 / pdf / stl / office） ———————————————————————————

/// Markdown 预览根容器。
pub(crate) const MD_PREVIEW_PANE: &str = "flex flex-col min-h-0 h-full box-border";

/// Markdown 滚动区（正文限宽居中）。
pub(crate) const MD_PREVIEW_SCROLL: &str = "flex-1 min-h-0 overflow-auto px-6 pt-5 pb-8 box-border dark:text-[#cccccc] [&_.ac-chat-md]:max-w-[52rem] [&_.ac-chat-md]:mx-auto";

/// 预览面板外壳（`select-none`：拖拽机身时不选中文字）。
pub(crate) const PREVIEW_PANE: &str = "relative flex flex-col min-h-0 h-full w-full box-border overflow-hidden bg-[var(--primary-color-1,#f1f5f9)] select-none data-[resizing=true]:cursor-grabbing dark:bg-[#141416]";

/// 同上，但允许选中（PDF / Office 阅读器）。
pub(crate) const PREVIEW_PANE_SELECTABLE: &str = "relative flex flex-col min-h-0 h-full w-full box-border overflow-hidden bg-[var(--primary-color-1,#f1f5f9)] select-auto dark:bg-[#141416]";

/// 舞台（白底）。
pub(crate) const PREVIEW_STAGE: &str = "flex-1 min-w-0 min-h-0 flex bg-white dark:bg-[#1c1c20]";

/// 舞台：手机模式（顶部留出机型 chip 高度 + 径向渐变底）。
pub(crate) const PREVIEW_STAGE_MOBILE: &str = "flex-1 min-w-0 min-h-0 flex items-center justify-center pt-14 px-5 pb-6 overflow-hidden bg-[radial-gradient(80%_60%_at_50%_0%,rgb(148_163_184/0.18),transparent_55%),var(--primary-color-1,#e2e8f0)] dark:bg-[radial-gradient(80%_60%_at_50%_0%,rgb(255_255_255/0.04),transparent_55%),#141416]";

/// 舞台：图片预览（居中 + 可滚动）。
pub(crate) const PREVIEW_STAGE_IMAGE: &str = "flex-1 min-w-0 min-h-0 flex items-center justify-center overflow-auto p-4 bg-[var(--primary-color-1,#f1f5f9)] dark:bg-[#1c1c20]";

/// 舞台：与自绘背景的查看器对齐（STL / Office）。
pub(crate) const PREVIEW_STAGE_PAPER: &str = "flex-1 min-w-0 min-h-0 flex bg-[#f8fafc] dark:bg-[#1c1c20]";

/// iframe 本体（撑满舞台）。
pub(crate) const PREVIEW_FRAME: &str = "flex-1 min-h-0 w-full h-full border-0 bg-white";

/// iframe 本体（机身内：跟随机身圆角、不吃 flex）。
pub(crate) const PREVIEW_FRAME_FILL: &str =
    "flex-none block w-full h-full border-0 bg-white rounded-[inherit]";

/// 机身占位（缩放后的真实尺寸由内联 style 给）。
pub(crate) const PREVIEW_DEVICE_SLOT: &str =
    "relative flex-none max-w-full max-h-full";

/// 手机机身：`border-width` 与 `files.rs` 的 `HTML_PREVIEW_PHONE_BEZEL` 保持一致。
pub(crate) const PREVIEW_DEVICE: &str = "group absolute top-0 left-0 box-content border border-primary-2 rounded-[3rem] bg-white shadow-[0_18px_48px_rgb(15_23_42/0.22)] origin-top-left dark:border-[#2e2e33] dark:shadow-[0_18px_48px_rgb(0_0_0/0.55)]";

/// 机身内层（裁掉 iframe 溢出的方角）。
pub(crate) const PREVIEW_DEVICE_BODY: &str =
    "w-full h-full overflow-hidden rounded-[calc(3rem-1px)] bg-white";

/// 四边手把：上。
pub(crate) const PREVIEW_HANDLE_N: &str =
    "absolute z-[2] top-[-4px] left-3 right-3 h-2 cursor-ns-resize";

/// 四边手把：下。
pub(crate) const PREVIEW_HANDLE_S: &str =
    "absolute z-[2] bottom-[-4px] left-3 right-3 h-2 cursor-ns-resize";

/// 四边手把：右。
pub(crate) const PREVIEW_HANDLE_E: &str =
    "absolute z-[2] top-3 right-[-4px] bottom-3 w-2 cursor-ew-resize";

/// 四边手把：左。
pub(crate) const PREVIEW_HANDLE_W: &str =
    "absolute z-[2] top-3 left-[-4px] bottom-3 w-2 cursor-ew-resize";

/// 四角：右上。
pub(crate) const PREVIEW_HANDLE_NE: &str = "absolute z-[2] w-[10px] h-[10px] border-[1.5px] border-brand rounded-[2px] bg-white opacity-0 group-hover:opacity-100 dark:border-[var(--brand,#60a5fa)] dark:bg-[#1c1c20] top-[-5px] right-[-5px] cursor-nesw-resize";

/// 四角：左上。
pub(crate) const PREVIEW_HANDLE_NW: &str = "absolute z-[2] w-[10px] h-[10px] border-[1.5px] border-brand rounded-[2px] bg-white opacity-0 group-hover:opacity-100 dark:border-[var(--brand,#60a5fa)] dark:bg-[#1c1c20] top-[-5px] left-[-5px] cursor-nwse-resize";

/// 四角：右下。
pub(crate) const PREVIEW_HANDLE_SE: &str = "absolute z-[2] w-[10px] h-[10px] border-[1.5px] border-brand rounded-[2px] bg-white opacity-0 group-hover:opacity-100 dark:border-[var(--brand,#60a5fa)] dark:bg-[#1c1c20] bottom-[-5px] right-[-5px] cursor-nwse-resize";

/// 四角：左下。
pub(crate) const PREVIEW_HANDLE_SW: &str = "absolute z-[2] w-[10px] h-[10px] border-[1.5px] border-brand rounded-[2px] bg-white opacity-0 group-hover:opacity-100 dark:border-[var(--brand,#60a5fa)] dark:bg-[#1c1c20] bottom-[-5px] left-[-5px] cursor-nesw-resize";

/// 拖拽期间的全屏遮罩（避免 iframe 吃掉鼠标事件）。
pub(crate) const PREVIEW_RESIZE_MASK: &str = "absolute inset-0 z-[12]";

/// 手机模式顶部的 chip 条容器。
pub(crate) const PREVIEW_PHONE_BAR: &str = "absolute top-[0.7rem] left-[0.7rem] z-[8] flex items-center gap-[0.35rem] max-w-[calc(100%-4.5rem)] pointer-events-none [&>*]:pointer-events-auto";

/// 胶囊按钮（机型选择 / 视口切换）；激活态交给 `data-active`。
pub(crate) const PREVIEW_CHIP: &str = "h-[1.85rem] max-w-full inline-flex items-center gap-[0.3rem] m-0 px-[0.6rem] border border-primary-2 rounded-full bg-[color-mix(in_srgb,var(--primary-color,#f8fafc)_88%,transparent)] text-[var(--text-color,#0f172a)] text-[0.72rem] leading-none whitespace-nowrap cursor-pointer shadow-[0_4px_12px_rgb(15_23_42/0.08)] hover:bg-primary data-[active=true]:border-[color-mix(in_srgb,var(--brand,#2563eb)_40%,transparent)] data-[active=true]:text-brand data-[active=true]:hover:bg-primary disabled:opacity-42 disabled:cursor-default disabled:shadow-none dark:border-[#2e2e33] dark:bg-[color-mix(in_srgb,#1c1c20_90%,transparent)] dark:text-[#e4e4e7] dark:shadow-[0_4px_12px_rgb(0_0_0/0.28)] dark:hover:bg-[#1c1c20] dark:data-[active=true]:text-[var(--brand,#93c5fd)] dark:data-[active=true]:border-[color-mix(in_srgb,var(--brand,#60a5fa)_40%,transparent)] [&_span]:overflow-hidden [&_span]:text-ellipsis";

/// 机型抽屉的遮罩。
pub(crate) const PREVIEW_DRAWER_MASK: &str = "absolute inset-0 z-[6] m-0 p-0 border-0 bg-[rgb(15_23_42/0.16)] cursor-pointer dark:bg-[rgb(0_0_0/0.35)]";

/// 机型抽屉（`data-open` 控制滑入）。
pub(crate) const PREVIEW_DRAWER: &str = "absolute top-0 bottom-0 left-0 z-[7] w-[15.25rem] flex flex-col gap-[0.2rem] pt-[3.15rem] px-[0.55rem] pb-[0.7rem] box-border border-r border-primary-2 bg-primary shadow-[8px_0_24px_rgb(15_23_42/0.1)] -translate-x-[105%] transition-transform duration-200 ease-out data-[open=true]:translate-x-0 dark:border-r-[#2e2e33] dark:bg-[#1c1c20] dark:shadow-[8px_0_24px_rgb(0_0_0/0.35)]";

/// 抽屉标题。
pub(crate) const PREVIEW_DRAWER_TITLE: &str = "mx-[0.35rem] mt-0 mb-[0.35rem] text-[0.7rem] font-semibold tracking-[0.04em] text-[var(--text-muted,#64748b)] dark:text-[#a1a1aa]";

/// 抽屉里的机型项（`data-active` 高亮）。
pub(crate) const PREVIEW_MODEL: &str = "flex flex-col items-start gap-[0.12rem] w-full m-0 px-2 py-[0.45rem] border-0 rounded-[0.5rem] bg-transparent text-inherit text-left cursor-pointer hover:bg-primary-2 data-[active=true]:bg-[color-mix(in_srgb,var(--brand,#2563eb)_14%,transparent)] data-[active=true]:hover:bg-[color-mix(in_srgb,var(--brand,#2563eb)_14%,transparent)] dark:hover:bg-[#2e2e33] dark:data-[active=true]:bg-[color-mix(in_srgb,var(--brand,#60a5fa)_18%,transparent)] dark:data-[active=true]:hover:bg-[color-mix(in_srgb,var(--brand,#60a5fa)_18%,transparent)]";

/// 机型名。
pub(crate) const PREVIEW_MODEL_NAME: &str = "text-[0.8rem] text-[var(--text-color,#0f172a)] dark:text-[#f4f4f5]";

/// 机型尺寸。
pub(crate) const PREVIEW_MODEL_SIZE: &str = "text-[0.68rem] text-[var(--text-muted,#64748b)] dark:text-[#a1a1aa]";

/// 右上角悬浮球容器。
pub(crate) const PREVIEW_FAB: &str =
    "absolute top-[0.7rem] right-[0.7rem] z-[9] flex flex-col items-end gap-[0.35rem]";

/// 悬浮球。
pub(crate) const PREVIEW_FAB_BALL: &str = "w-9 h-9 inline-flex items-center justify-center m-0 p-0 border border-primary-2 rounded-full bg-[color-mix(in_srgb,var(--primary-color,#f8fafc)_92%,transparent)] text-[var(--text-color,#0f172a)] cursor-pointer shadow-[0_8px_20px_rgb(15_23_42/0.14)] hover:text-brand hover:border-[color-mix(in_srgb,var(--brand,#2563eb)_35%,transparent)] dark:border-[#2e2e33] dark:bg-[color-mix(in_srgb,#1c1c20_92%,transparent)] dark:text-[#e4e4e7] dark:shadow-[0_8px_20px_rgb(0_0_0/0.35)] dark:hover:text-[var(--brand,#93c5fd)] dark:hover:border-[color-mix(in_srgb,var(--brand,#60a5fa)_40%,transparent)]";

/// 次级悬浮球（还原尺寸）：图标用黑色。
pub(crate) const PREVIEW_FAB_BALL_SECONDARY: &str = "w-9 h-9 inline-flex items-center justify-center m-0 p-0 border border-primary-2 rounded-full bg-[color-mix(in_srgb,var(--primary-color,#f8fafc)_92%,transparent)] text-black cursor-pointer shadow-[0_8px_20px_rgb(15_23_42/0.14)] hover:border-[color-mix(in_srgb,var(--brand,#2563eb)_35%,transparent)] dark:border-[#2e2e33] dark:bg-[color-mix(in_srgb,#1c1c20_92%,transparent)] dark:text-[#e4e4e7] dark:shadow-[0_8px_20px_rgb(0_0_0/0.35)] dark:hover:border-[color-mix(in_srgb,var(--brand,#60a5fa)_40%,transparent)]";

/// 图片本体：棋盘格衬透明像素。
pub(crate) const PREVIEW_IMAGE: &str = "block max-w-full max-h-full w-auto h-auto object-contain shadow-[0_8px_28px_rgb(15_23_42/0.18)] dark:shadow-[0_8px_28px_rgb(0_0_0/0.55)] bg-white bg-[image:linear-gradient(45deg,#c8c8c8_25%,transparent_25%),linear-gradient(-45deg,#c8c8c8_25%,transparent_25%),linear-gradient(45deg,transparent_75%,#c8c8c8_75%),linear-gradient(-45deg,transparent_75%,#c8c8c8_75%)] [background-size:16px_16px] [background-position:0_0,0_8px,8px_-8px,-8px_0]";

/// 音频面板：强调色 / 波形底色用自定义属性，暗色主题换一套（原 `html[data-theme="dark"]` 覆盖）。
pub(crate) const AUDIO_PANE: &str = "relative flex flex-col min-h-0 h-full w-full box-border overflow-hidden select-none bg-[var(--primary-color-1,#f1f5f9)] [--ac-audio-accent:#bac5db] [--ac-audio-track-bg:color-mix(in_srgb,var(--ac-audio-accent)_16%,#f4f6f9)] [--ac-audio-track-fill:var(--ac-audio-accent)] dark:[--ac-audio-accent:#8b97b3] dark:[--ac-audio-track-bg:color-mix(in_srgb,var(--ac-audio-accent)_22%,#1e1e22)]";

/// 音频舞台（径向渐变底）。
pub(crate) const AUDIO_STAGE: &str = "flex-1 min-w-0 min-h-0 flex items-center justify-center overflow-auto px-4 py-6 bg-[radial-gradient(120%_80%_at_50%_0%,color-mix(in_srgb,var(--ac-audio-accent)_22%,transparent),transparent_62%),var(--primary-color-1,#f8fafc)] dark:bg-[radial-gradient(120%_80%_at_50%_0%,color-mix(in_srgb,var(--ac-audio-accent)_16%,transparent),transparent_62%),#141416]";

/// 原生 `<audio>`（隐藏，只用自绘控件）。
pub(crate) const AUDIO_EL: &str = "hidden";

/// 音频卡片。
pub(crate) const AUDIO_CARD: &str = "box-border w-[min(36rem,100%)] flex flex-col items-center gap-[1.1rem] px-[1.4rem] pt-[1.6rem] pb-[1.35rem] rounded-2xl border border-primary-2 bg-[color-mix(in_srgb,var(--primary-color,#fff)_88%,transparent)] shadow-[0_12px_36px_rgb(15_23_42/0.08)] dark:border-[#2a2a2e] dark:bg-[#1a1a1d] dark:shadow-[0_12px_36px_rgb(0_0_0/0.35)]";

/// 音频标题。
pub(crate) const AUDIO_TITLE: &str = "max-w-full text-[0.95rem] font-[650] text-[var(--text-primary,#2a2f37)] text-center truncate";

/// 播放按钮。
pub(crate) const AUDIO_PLAY: &str = "box-border w-[3.6rem] h-[3.6rem] border-0 rounded-full inline-flex items-center justify-center text-white bg-[var(--ac-audio-accent)] cursor-pointer appearance-none shadow-[0_8px_22px_color-mix(in_srgb,var(--ac-audio-accent)_40%,transparent)] hover:brightness-[1.04] active:scale-[0.97]";

/// 进度条轨道。
pub(crate) const AUDIO_TRACK: &str = "box-border w-full h-[0.35rem] rounded-full bg-[var(--ac-audio-track-bg)] overflow-hidden cursor-pointer";

/// 进度条填充（宽度由 JS 每帧直写，禁用过渡）。
pub(crate) const AUDIO_TRACK_FILL: &str = "h-full w-0 rounded-[inherit] bg-[var(--ac-audio-track-fill)] transition-none pointer-events-none will-change-[width]";

/// 时间元信息。
pub(crate) const AUDIO_META: &str = "w-full flex justify-between text-[0.72rem] tabular-nums text-[var(--text-secondary,#64758b)]";

/// 加载失败提示。
pub(crate) const AUDIO_ERROR: &str = "w-full text-[0.78rem] text-[var(--ac-chat-error-icon,#dc2626)] text-center";

// ——————————————————————————— 聊天栏外壳 ———————————————————————————

/// 聊天栏（含 premium-chat 的表面）。`group/exp` 驱动中栏收起后的放宽；拖拽时禁过渡。
pub(crate) const CHAT: &str = "group/exp relative flex flex-col min-w-[280px] max-w-[560px] flex-[0_0_auto] border-l border-line bg-primary overflow-x-hidden shadow-[-2px_4px_20px_var(--shadow-color)] [--ac-chat-body-fs:0.9rem] transition-[flex,width,max-width] duration-[0.22s] ease-out group-data-[resizing=true]/root:[transition:none]! dark:border-l-transparent dark:before:[content:''] dark:before:absolute dark:before:top-0 dark:before:left-0 dark:before:bottom-0 dark:before:w-px dark:before:z-[6] dark:before:pointer-events-none dark:before:opacity-[var(--ac-sand-opacity)] dark:before:bg-[var(--ac-sand-grad-y)] data-[menus=true]:overflow-visible data-[menus=true]:z-80";

/// 聊天栏已隐藏。
pub(crate) const CHAT_HIDDEN: &str = "flex-[0_0_0]! w-0! min-w-0! max-w-0! opacity-0 overflow-hidden pointer-events-none border-l-0 border-l-transparent";

/// 聊天栏主体（web 隐藏聊天栏后顶栏改挂根节点的浮层由别处常量承担）。
pub(crate) const CHAT_WORKSPACE: &str = "flex-1 min-h-0 flex flex-row items-stretch group-data-[tbsp=true]/exp:pt-[calc(1.5rem+34px)]";

/// 聊天栏内容列。
pub(crate) const CHAT_MAIN: &str = "flex-1 min-w-0 min-h-0 flex flex-col";

/// 右侧聊天栏宽度拖拽热区。
pub(crate) const CHAT_RESIZER: &str = "absolute top-0 bottom-0 -left-[5px] w-[10px] cursor-col-resize z-[6] bg-transparent transition-[background] duration-150 ease-out hover:bg-[linear-gradient(90deg,var(--ac-color-4285f42e),var(--ac-color-4285f42e)_60%,transparent)]";

/// 聊天栏头部。
pub(crate) const CHAT_HEADER: &str = "px-5 py-4 border-b border-primary-2 shrink-0 bg-primary backdrop-blur-[16px]";

/// 头部图标。
pub(crate) const CHAT_HEADER_ICON: &str = "shrink-0";

/// 正文列（左上柔光 + 竖向渐变底）。
pub(crate) const CHAT_BODY: &str = "flex-1 min-h-0 overflow-hidden p-0 flex flex-col items-stretch gap-0 bg-[radial-gradient(circle_at_24%_12%,var(--ac-color-4285f42e),transparent_24%),linear-gradient(180deg,var(--primary-color-1),var(--primary-color-1))]";

/// 线程窗格（相对定位，承载左侧导航短条）。
pub(crate) const CHAT_THREAD_PANE: &str = "relative flex-1 min-w-0 min-h-0 flex flex-col";

/// 左缘提问导航短条容器。
pub(crate) const CHAT_NAV: &str = "absolute left-[3px] top-1/2 -translate-y-1/2 z-[4] flex flex-col items-start gap-[7px] py-[2px]";

/// 导航短条（`data-current` 深色）。
pub(crate) const CHAT_NAV_DOT: &str = "w-[15px] h-1 p-0 border-0 rounded-[2px] bg-[#d5d5d5] cursor-pointer transition-[background] duration-[0.12s] ease-out hover:bg-[#a8a8a8] data-[current=true]:bg-[#1a1a1a]";

/// 消息线程滚动区。
pub(crate) const CHAT_THREAD: &str = "min-w-0 min-h-0 flex-1 overflow-y-auto overflow-x-hidden flex flex-col gap-[2.25rem] box-border py-6 px-4 group-data-[exp=true]/exp:px-6";

/// 线程与输入区之间的零高槽位（回到底部按钮贴在输入区上缘）。
pub(crate) const CHAT_JUMP_HOST: &str = "group/jh relative z-[6] h-0 overflow-visible pointer-events-none";

/// 回到底部按钮。
pub(crate) const CHAT_JUMP_BOTTOM: &str = "absolute left-1/2 bottom-[0.14rem] -translate-x-1/2 z-[6] w-[1.55rem] h-[1.55rem] min-w-[1.55rem] min-h-[1.55rem] p-0 border rounded-full border-primary-2 inline-flex items-center justify-center text-secondary-3 bg-primary shadow-[0_8px_20px_var(--shadow-color)] cursor-pointer pointer-events-auto transition-[background,color,border-color,box-shadow,transform] duration-150 ease-out group-data-[hidden=true]/jh:opacity-0 group-data-[hidden=true]/jh:invisible group-data-[hidden=true]/jh:pointer-events-none hover:text-secondary-1 hover:bg-primary-1 hover:border-border hover:shadow-[0_12px_24px_var(--shadow-color)] hover:-translate-y-px focus-visible:text-secondary-1 focus-visible:bg-primary-1 focus-visible:border-border focus-visible:shadow-[0_12px_24px_var(--shadow-color)] focus-visible:-translate-y-px focus-visible:outline-2 focus-visible:outline-border-hover focus-visible:outline-offset-2 motion-reduce:hover:translate-y-0 motion-reduce:focus-visible:translate-y-0";

// ——————————————————————————— 消息与气泡 ———————————————————————————

/// 消息行。
pub(crate) const CHAT_MESSAGE: &str = "flex items-start gap-3 min-w-0 max-w-full";

/// 用户消息行（靠右）。
pub(crate) const CHAT_MESSAGE_USER: &str = "justify-end";

/// 助手消息行（靠左）。
pub(crate) const CHAT_MESSAGE_AI: &str = "justify-start";

/// 气泡基底（宽度由内容决定；`overflow-wrap` 保证上限处换行但不把 min-content 压成单字）。
pub(crate) const CHAT_BUBBLE: &str = "ac-chat-bubble w-max max-w-[min(620px,100%)] box-border px-4 py-3 rounded-[0.7rem] text-[length:var(--ac-chat-body-fs,0.9rem)] leading-[1.65] font-[650] shadow-[0_10px_24px_var(--shadow-color)] [overflow-wrap:break-word] [word-break:normal] [&_.ac-chat-md_h1]:text-[1.15rem] [&_.ac-chat-md_h2]:text-[1.08rem] [&_.ac-chat-md_h3]:text-[1.01rem] [&_.ac-chat-md_h4]:text-[1.01rem] [&_.ac-chat-md_pre]:text-[0.86rem] [&_.ac-chat-code-pre]:text-[0.86rem] [&_.ac-chat-md_table]:text-[0.88rem]";

/// 助手气泡：无壳透明底。
pub(crate) const CHAT_BUBBLE_AI: &str = "ac-chat-bubble-ai max-w-full text-[var(--ac-chat-ai-fg)] [&_.ac-chat-md_h1]:text-[var(--ac-chat-ai-fg-strong)] [&_.ac-chat-md_h2]:text-[var(--ac-chat-ai-fg-strong)] [&_.ac-chat-md_h3]:text-[var(--ac-chat-ai-fg-strong)] [&_.ac-chat-md_h4]:text-[var(--ac-chat-ai-fg-strong)] [&_.ac-chat-md_strong]:text-[var(--ac-chat-ai-fg-strong)] bg-transparent border-0 shadow-none pt-[0.35rem] pr-0 pb-[0.15rem] pl-0 rounded-none animate-[ac-chat-bubble-enter_0.28s_cubic-bezier(0.22,1,0.36,1)_both]";

/// 用户气泡。
pub(crate) const CHAT_BUBBLE_USER: &str = "ac-chat-bubble-user text-text-primary bg-[var(--ac-chat-bubble-user-bg)] border-0 group-data-[exp=true]/exp:max-w-[min(1000px,calc(100%-20px))]";

/// 用户气泡编辑态。
pub(crate) const CHAT_BUBBLE_USER_EDITING: &str = "relative z-[2] w-[min(620px,calc(100%-20px))] max-w-[min(620px,calc(100%-20px))] pt-[0.55rem] pr-[0.65rem] pb-[0.5rem] pl-[0.65rem]";

/// 用户消息正文（innerHTML 生成，保留 pre-wrap）。
pub(crate) const CHAT_BUBBLE_USER_TEXT: &str = "inline whitespace-pre-wrap [overflow-wrap:break-word] [word-break:normal]";

/// 助手消息纵向栈（悬停浮现操作行）。
pub(crate) const CHAT_AI_STACK: &str = "group/ai shrink-0 w-fit max-w-[min(720px,calc(100%-20px))] flex flex-col items-start pb-[1.75rem] -mb-[1.75rem] group-data-[exp=true]/exp:max-w-[min(1000px,calc(100%-20px))]";

/// 用户消息纵向栈（悬停浮现操作行）。
pub(crate) const CHAT_USER_STACK: &str = "group/user shrink-0 w-fit max-w-[min(720px,calc(100%-20px))] flex flex-col items-end pb-[1.75rem] -mb-[1.75rem] group-data-[exp=true]/exp:max-w-[min(1000px,calc(100%-20px))]";

/// 助手消息操作行（`data-error` 常显）。
pub(crate) const CHAT_AI_ACTIONS: &str = "group/actions min-h-[1.55rem] inline-flex items-center gap-[0.35rem] pt-[0.35rem] opacity-0 -translate-y-[0.18rem] pointer-events-none transition-[opacity,transform] duration-[0.16s] ease-out group-hover/ai:opacity-100 group-hover/ai:translate-y-0 group-hover/ai:pointer-events-auto group-focus-within/ai:opacity-100 group-focus-within/ai:translate-y-0 group-focus-within/ai:pointer-events-auto data-[error=true]:opacity-100 data-[error=true]:translate-y-0 data-[error=true]:pointer-events-auto";

/// 用户消息操作行。
pub(crate) const CHAT_USER_ACTIONS: &str = "min-h-[1.55rem] inline-flex items-center justify-end gap-[0.35rem] pt-[0.35rem] opacity-0 -translate-y-[0.18rem] pointer-events-none transition-[opacity,transform] duration-[0.16s] ease-out group-hover/user:opacity-100 group-hover/user:translate-y-0 group-hover/user:pointer-events-auto group-focus-within/user:opacity-100 group-focus-within/user:translate-y-0 group-focus-within/user:pointer-events-auto";

/// 操作行里的图标按钮。
pub(crate) const CHAT_AI_ACTION_BTN: &str = "w-[1.65rem] min-w-[1.65rem] h-[1.65rem] inline-flex items-center justify-center gap-[0.28rem] p-0 border-0 rounded-[0.45rem] text-secondary-3 bg-transparent shadow-none cursor-pointer transition-[color,transform] duration-[0.16s] ease-out hover:text-brand hover:-translate-y-px focus-visible:text-brand focus-visible:-translate-y-px group-data-[error=true]/actions:w-auto group-data-[error=true]/actions:px-[0.52rem]";

/// 操作行图标。
pub(crate) const CHAT_AI_ACTION_ICON: &str = "shrink-0";

/// 操作行文字。
pub(crate) const CHAT_AI_ACTION_TEXT: &str = "text-[0.76rem] font-semibold leading-none whitespace-nowrap";

// ——————————————————————————— 气泡内编辑 ———————————————————————————

pub(crate) const CHAT_BUBBLE_EDIT: &str = "flex flex-col gap-[0.35rem] min-w-0";

pub(crate) const CHAT_BUBBLE_EDIT_ACTIONS: &str = "flex items-center justify-end gap-[0.35rem] pt-[0.1rem]";

pub(crate) const CHAT_BUBBLE_EDIT_UPLOAD: &str = "relative inline-flex items-center justify-center w-[1.55rem] h-[1.55rem] shrink-0 text-secondary-2 cursor-pointer transition-[color,transform] duration-150 hover:text-secondary-1 hover:-translate-y-px";

pub(crate) const CHAT_BUBBLE_EDIT_SEND: &str = "flex-none w-[1.85rem] min-w-[1.85rem] h-[1.85rem] min-h-[1.85rem] mt-0 self-auto rounded-full transition-[background,color,filter,box-shadow,transform] duration-150 not-disabled:hover:-translate-y-px not-disabled:focus-visible:-translate-y-px";

// ——————————————————————————— 发送钮 ———————————————————————————

/// 发送钮基底。
pub(crate) const CHAT_SEND_BTN: &str = "flex-none w-[2.25rem] min-w-[2.25rem] h-[2.25rem] min-h-[2.25rem] mt-auto self-end p-0 border-0 rounded-full inline-flex items-center justify-center text-white bg-[var(--search-button)] shadow-none cursor-default pointer-events-auto transition-[background,color,filter,box-shadow,transform] duration-150 disabled:opacity-45 disabled:cursor-not-allowed not-disabled:hover:bg-[var(--search-button-hover)] not-disabled:hover:text-white not-disabled:hover:[filter:none] not-disabled:hover:-translate-y-px not-disabled:hover:cursor-pointer not-disabled:focus-visible:bg-[var(--search-button-hover)] not-disabled:focus-visible:text-white not-disabled:focus-visible:[filter:none] not-disabled:focus-visible:-translate-y-px";

/// 草稿非空：灰蓝底。
pub(crate) const CHAT_SEND_READY: &str = "text-primary bg-[var(--ac-chat-send-ready)] shadow-[0_14px_30px_var(--shadow-color)] cursor-pointer not-disabled:hover:bg-[var(--ac-chat-send-ready-hover)] not-disabled:hover:text-white not-disabled:hover:[filter:none] not-disabled:focus-visible:bg-[var(--ac-chat-send-ready-hover)] not-disabled:focus-visible:text-white not-disabled:focus-visible:[filter:none]";

/// 停止生成。
pub(crate) const CHAT_SEND_PAUSE: &str = "bg-[var(--border-color)] cursor-pointer hover:bg-[var(--ac-chat-send-ready-hover)] hover:text-white focus-visible:bg-[var(--ac-chat-send-ready-hover)] focus-visible:text-white";

/// 发送图标。
pub(crate) const CHAT_SEND_ICON: &str = "shrink-0";

// ——————————————————————————— 工具轨迹 / 子任务卡（thinking.rs） ———————————————————————————

pub(crate) const CHAT_TRACE: &str = "w-full mt-[0.1rem] mr-0 mb-[0.55rem] ml-0";

pub(crate) const CHAT_TRACE_SUMMARY: &str = "inline-flex items-center gap-[0.3rem] max-w-full pt-[0.1rem] pr-[0.15rem] pb-[0.1rem] pl-0 border-0 bg-transparent text-text-secondary text-[0.78rem] font-semibold leading-[1.4] cursor-pointer hover:text-text-primary";

pub(crate) const CHAT_TRACE_SUMMARY_LABEL: &str = "min-w-0";

pub(crate) const CHAT_TRACE_CHEVRON: &str = "inline-flex shrink-0 opacity-70";

pub(crate) const CHAT_TRACE_BODY: &str = "mt-[0.25rem] pl-[0.05rem]";

pub(crate) const CHAT_TRACE_ACTIVITY: &str = "mb-[0.2rem] text-text-secondary text-[0.74rem] font-[550] leading-[1.45] opacity-90";

pub(crate) const CHAT_TRACE_ACTIVITY_LABEL: &str = "";

pub(crate) const CHAT_TRACE_LIST: &str = "flex flex-col gap-[0.12rem]";

/// 轨迹行：`data-thought` 纵排、`data-running` 抬高动词对比度。
pub(crate) const CHAT_TRACE_ROW: &str = "group/row flex flex-wrap items-baseline gap-x-[0.45rem] gap-y-[0.35rem] min-w-0 text-[0.74rem] leading-[1.45] text-text-secondary data-[thought=true]:flex-col data-[thought=true]:items-start data-[thought=true]:gap-[0.2rem]";

pub(crate) const CHAT_TRACE_THOUGHT_BTN: &str = "group/thbtn inline-flex items-center gap-[0.25rem] p-0 border-0 bg-transparent text-[inherit] [font:inherit] cursor-pointer";

pub(crate) const CHAT_TRACE_MINI_CHEVRON: &str = "inline-flex opacity-55";

pub(crate) const CHAT_TRACE_THOUGHT_HEAD: &str = "inline-flex items-center gap-[0.35rem]";

/// 思维正文：`data-live` 限高滚动、`data-snip` 三行截断。
pub(crate) const CHAT_TRACE_THOUGHT_BODY: &str = "m-0 pt-[0.25rem] pr-0 pb-[0.15rem] pl-[0.05rem] max-w-full whitespace-pre-wrap [word-break:break-word] text-text-secondary [font-family:inherit] text-[0.8rem] leading-[1.5] opacity-85 data-[live=true]:max-h-[7.5rem] data-[live=true]:overflow-y-auto data-[snip=true]:line-clamp-3 data-[snip=true]:opacity-72";

pub(crate) const CHAT_TRACE_VERB: &str = "shrink-0 text-[color-mix(in_srgb,var(--secondary-color-1)_78%,var(--secondary-color-3))] font-semibold group-hover/thbtn:text-text-primary data-[muted=true]:text-text-secondary data-[muted=true]:font-[550] group-data-[running=true]/row:opacity-95";

pub(crate) const CHAT_TRACE_PRIMARY: &str = "min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-text-secondary [font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace] text-[0.72rem] opacity-92";

/// 链接形态的主参数（button）。
pub(crate) const CHAT_TRACE_PRIMARY_LINK: &str = "min-w-0 overflow-hidden text-ellipsis whitespace-nowrap p-0 border-0 border-b border-b-[color-mix(in_srgb,var(--ac-color-2563eb)_45%,transparent)] bg-transparent text-ac-blue [font:inherit] [font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace] text-[0.72rem] leading-[inherit] text-left cursor-pointer opacity-100 hover:text-[color-mix(in_srgb,var(--ac-color-2563eb)_75%,var(--secondary-color-1))] hover:[border-bottom-color:currentColor]";

pub(crate) const CHAT_TRACE_SECONDARY: &str = "min-w-0 text-text-secondary [font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace] text-[0.7rem] opacity-65";

pub(crate) const CHAT_TRACE_WAITING: &str = "inline-flex items-center gap-[0.4rem] min-h-[1.1rem]";

pub(crate) const CHAT_TRACE_PULSE: &str = "w-[0.38rem] h-[0.38rem] rounded-full bg-text-secondary opacity-55 animate-[ac-chat-trace-pulse_1.05s_ease-in-out_infinite]";

/// 子任务卡（`data-running` 描边变蓝）。
pub(crate) const CHAT_SUBTASK_CARD: &str = "flex flex-col items-start gap-[0.22rem] mt-[0.18rem] mr-0 mb-[0.28rem] ml-0 px-[0.62rem] py-[0.5rem] border rounded-[0.55rem] border-primary-2 bg-[linear-gradient(180deg,var(--primary-color),var(--primary-color-3))] shadow-[0_1px_4px_var(--shadow-color)] data-[running=true]:border-[color-mix(in_srgb,var(--ac-color-2563eb,#2563eb)_35%,var(--primary-color-2))]";

pub(crate) const CHAT_SUBTASK_HEAD: &str = "flex items-center gap-[0.4rem] w-full min-w-0";

pub(crate) const CHAT_SUBTASK_BADGE: &str = "shrink-0 px-[0.35rem] py-[0.08rem] rounded-full bg-[color-mix(in_srgb,var(--ac-color-2563eb,#2563eb)_16%,transparent)] text-[var(--ac-color-2563eb,#2563eb)] text-[0.62rem] font-bold tracking-[0.02em]";

pub(crate) const CHAT_SUBTASK_TITLE: &str = "min-w-0 flex-1 text-[0.78rem] font-bold text-secondary-1";

pub(crate) const CHAT_SUBTASK_STATUS: &str = "shrink-0 text-[0.66rem] font-semibold text-secondary-3";

pub(crate) const CHAT_SUBTASK_TEXT: &str = "max-w-full text-[0.7rem] leading-[1.4] text-secondary-3 [word-break:break-word]";

pub(crate) const CHAT_SUBTASK_SUMMARY: &str = "max-w-full text-[0.7rem] leading-[1.4] text-secondary-2 [word-break:break-word]";

pub(crate) const CHAT_SUBTASK_OPEN: &str = "mt-[0.08rem] px-[0.45rem] py-[0.18rem] border rounded-[0.4rem] border-primary-2 bg-primary text-secondary-1 text-[0.7rem] font-[650] cursor-pointer hover:bg-primary-1";

// ——————————————————————————— 输入区与工具栏（P5-2） ———————————————————————————

/// 输入区外壳（顶部 1px 分隔；暗色改银河横线并隐藏边框）。
pub(crate) const CHAT_INPUT_WRAP: &str = "relative z-[5] box-border w-full overflow-x-hidden border-t border-primary-2 bg-primary shrink-0 pointer-events-auto group-data-[menus=true]/exp:overflow-visible group-data-[menus=true]/exp:overflow-x-visible dark:border-t-transparent dark:before:[content:''] dark:before:absolute dark:before:left-0 dark:before:right-0 dark:before:top-0 dark:before:h-px dark:before:z-[6] dark:before:pointer-events-none dark:before:opacity-[var(--ac-sand-opacity)] dark:before:bg-[var(--ac-sand-grad-x)]";

/// 输入壳（内容层 + 工具条的圆角容器）。
pub(crate) const CHAT_INPUT_SHELL: &str = "relative z-[6] box-border w-full min-h-[6.5rem] rounded-[0.75rem] border-0 bg-primary flex items-stretch gap-[0.45rem] pt-[0.35rem] pr-[0.35rem] pb-[0.35rem] pl-[0.55rem] shadow-[0_8px_22px_var(--shadow-color)] pointer-events-auto";

/// 输入区右列（CE + 附件行 + 工具条）。
pub(crate) const CHAT_COMPOSER_STACK: &str = "flex-1 min-w-0 min-h-0 flex flex-col gap-[0.25rem] self-stretch";

/// 工具条（模型 / 模式 / 上传）。
pub(crate) const CHAT_COMPOSER_TOOLBAR: &str = "flex flex-wrap items-center gap-x-[0.65rem] gap-y-[0.45rem] pb-[0.1rem]";

/// 单 contenteditable（文字与 atomic chip 同行混排）。类名保留供 JS 桥与 `::before` 占位符使用。
pub(crate) const CHAT_COMPOSER_CE: &str = "ac-chat-composer-ce relative block w-full min-w-0 min-h-[4rem] max-h-[10rem] overflow-x-hidden overflow-y-auto box-border pt-[0.15rem] pr-[0.1rem] pb-0 pl-[0.1rem] outline-none whitespace-pre-wrap [overflow-wrap:anywhere] [word-break:break-word] [caret-color:var(--secondary-color-1)] leading-[1.35] text-[length:var(--ac-chat-body-fs,0.9rem)] text-secondary-1 [font-family:-apple-system,BlinkMacSystemFont,'PingFang_SC','Hiragino_Sans_GB','STHeiti','Heiti_SC','Microsoft_YaHei','Noto_Sans_CJK_SC',system-ui,sans-serif] pointer-events-auto";

/// 工具条胶囊基底（模型 / 模式 / 上传共用）。
pub(crate) const CHAT_TOOLBAR_PILL: &str = "box-border appearance-none rounded-full border border-[var(--ac-chat-toolbar-pill-border)] bg-[var(--ac-chat-toolbar-pill-bg)] shadow-[var(--ac-chat-toolbar-pill-shadow)] cursor-pointer transition-[border-color,box-shadow,background,background-image,color,transform] duration-150 ease-out hover:border-[color-mix(in_srgb,var(--ac-chat-toolbar-pill-border)_55%,var(--border-color-hover))] hover:shadow-[0_3px_8px_#0f172a14] active:scale-[0.98]";

/// Agent 模式触发钮（图标 + 文字胶囊；`data-open` 展开）。
pub(crate) const CHAT_AGENT_MODE_BTN: &str = "box-border appearance-none inline-flex items-center justify-center gap-[0.35rem] min-h-[2rem] min-w-0 w-auto h-auto pt-[0.28rem] pr-[0.7rem] pb-[0.28rem] pl-[0.55rem] shrink-0 text-secondary-1 text-[0.7rem] font-[650] leading-[1.2] tracking-[-0.02em] not-disabled:hover:text-[var(--border-color-dark)] data-[open=true]:text-[var(--border-color-dark)]";

pub(crate) const CHAT_AGENT_MODE_BTN_ICON: &str = "shrink-0";

pub(crate) const CHAT_AGENT_MODE_BTN_LABEL: &str = "shrink-0 whitespace-nowrap";

/// Agent 模式面板宽度微调。
pub(crate) const CHAT_AGENT_MODE_SHEET: &str = "min-w-[14.5rem] p-[0.35rem]";

/// Agent 模式选项行（`data-active` 选中）。
pub(crate) const CHAT_AGENT_MODE_OPT: &str = "group/opt relative box-border flex items-center gap-[0.55rem] w-full m-0 pt-[0.48rem] pr-[2rem] pb-[0.48rem] pl-[0.55rem] border-0 rounded-[0.5rem] bg-transparent text-left cursor-pointer text-secondary-1 transition-[background] duration-[0.12s] ease-out hover:bg-primary-1 data-[active=true]:bg-primary-1 [&:not(:first-child)]:mt-[0.08rem]";

pub(crate) const CHAT_AGENT_MODE_OPT_ICON: &str = "inline-flex items-center justify-center w-[1.1rem] h-[1.1rem] shrink-0 text-secondary-2 group-hover/opt:text-secondary-1 group-data-[active=true]/opt:text-secondary-1";

pub(crate) const CHAT_AGENT_MODE_OPT_LABEL: &str = "flex-1 min-w-0 text-[0.78rem] font-[650] tracking-[-0.02em]";

pub(crate) const CHAT_AGENT_MODE_OPT_CHECK: &str = "absolute right-[0.55rem] top-1/2 -translate-y-1/2 text-[var(--ac-color-312e81)] pointer-events-none";

/// 模型 / 模式触发器的外层（相对定位承载上拉面板；`group/mw` + `data-model` 驱动模型面板变体）。
pub(crate) const CHAT_MODE_WRAP: &str = "group/mw relative shrink-0 z-[4]";

/// 模式 / 模型触发钮（`data-open` 展开）。
pub(crate) const CHAT_MODE_TRIGGER: &str = "group/trg box-border appearance-none inline-flex items-center justify-center gap-[0.35rem] min-h-[2rem] pt-[0.28rem] pr-[0.65rem] pb-[0.28rem] pl-[0.75rem] text-[0.7rem] font-[650] leading-[1.2] tracking-[-0.02em] text-secondary-1 hover:text-[var(--border-color-dark)] data-[open=true]:border-[color-mix(in_srgb,var(--ac-chat-toolbar-pill-border)_40%,var(--border-color-hover))] data-[open=true]:bg-[linear-gradient(180deg,#ffffff_0%,#d5deec_100%)] data-[open=true]:text-secondary-1 data-[open=true]:shadow-[0_2px_8px_#0f172a14]";

pub(crate) const CHAT_MODE_TRIGGER_LABEL: &str = "max-w-[8.5rem] overflow-hidden text-ellipsis whitespace-nowrap";

pub(crate) const CHAT_MODE_TRIGGER_CHEVRON: &str = "shrink-0 opacity-72 group-data-[open=true]/trg:opacity-95";

/// 上拉面板遮罩（透明，仅收点击）。
pub(crate) const CHAT_MODE_SHEET_BACKDROP: &str = "fixed inset-0 z-[5000] bg-transparent";

/// 上拉面板（模式 / 模型共用；`group-data-[model=true]/mw` 提供模型面板的限高滚动）。
pub(crate) const CHAT_MODE_SHEET: &str = "absolute bottom-full left-0 z-[5001] min-w-[14.5rem] mb-[0.45rem] p-[0.4rem] rounded-[0.65rem] border border-primary-2 bg-primary backdrop-blur-[12px] shadow-[0_-2px_4px_var(--shadow-color),0_18px_40px_-8px_var(--shadow-color)] animate-[ac-chat-mode-sheet-in_0.2s_cubic-bezier(0.22,1,0.36,1)] group-data-[model=true]/mw:max-w-[min(22rem,calc(100vw-2rem))] group-data-[model=true]/mw:max-h-[min(26rem,62vh)] group-data-[model=true]/mw:overflow-x-hidden group-data-[model=true]/mw:overflow-y-auto";

/// 面板分组（组间分隔线）。
pub(crate) const CHAT_SHEET_GROUP: &str = "m-0 [&:not(:first-child)]:mt-[0.2rem] [&:not(:first-child)]:pt-[0.2rem] [&:not(:first-child)]:border-t [&:not(:first-child)]:border-t-[color-mix(in_srgb,var(--primary-color-2)_78%,transparent)]";

/// 分组头（`group/gh` + `data-open`）。
pub(crate) const CHAT_SHEET_GROUP_HEAD: &str = "group/gh box-border w-full m-0 px-[0.5rem] py-[0.42rem] border-0 rounded-[0.45rem] flex items-center gap-[0.35rem] bg-transparent text-secondary-2 cursor-pointer text-left appearance-none hover:bg-primary-1 group-data-[open=true]/gh:text-secondary-1";

pub(crate) const CHAT_SHEET_GROUP_CHEVRON: &str = "flex-none inline-flex opacity-72";

pub(crate) const CHAT_SHEET_GROUP_TITLE: &str = "flex-[1_1_auto] min-w-0 text-[0.72rem] font-bold tracking-[0.01em] overflow-hidden text-ellipsis whitespace-nowrap";

pub(crate) const CHAT_SHEET_GROUP_COUNT: &str = "flex-none text-[0.62rem] font-[650] [font-variant-numeric:tabular-nums] text-secondary-3";

pub(crate) const CHAT_SHEET_GROUP_BODY: &str = "pt-[0.1rem] pb-[0.15rem] px-0";

/// 选项行（`group/row` + `data-active` / `data-custom`）。
pub(crate) const CHAT_SHEET_OPT_ROW: &str = "group/row relative flex items-stretch w-full hover:bg-primary-1 hover:rounded-[0.5rem] data-[active=true]:bg-primary-1 data-[active=true]:rounded-[0.5rem] data-[custom=true]:flex-wrap [&:not(:first-child)]:mt-[0.12rem]";

pub(crate) const CHAT_SHEET_OPT_MAIN: &str = "relative flex items-stretch w-full";

/// 选项主体（`group/opt` + `data-active`；模型面板的选中态经 `group/mw` 微调）。
pub(crate) const CHAT_SHEET_OPT: &str = "group/opt relative box-border flex flex-col items-start gap-[0.12rem] w-full m-0 px-[0.65rem] py-[0.55rem] border-0 rounded-[0.5rem] bg-transparent text-left cursor-pointer text-secondary-1 transition-[background] duration-[0.12s] ease-out hover:bg-primary-1 group-hover/row:bg-transparent group-data-[active=true]/row:bg-transparent data-[active=true]:bg-transparent data-[active=true]:shadow-none data-[custom=true]:pr-[2.1rem] group-data-[model=true]/mw:data-[active=true]:pr-[2rem] group-data-[model=true]/mw:group-data-[custom=true]/row:data-[active=true]:pr-[3.5rem]";

pub(crate) const CHAT_SHEET_TITLE_ROW: &str = "inline-flex items-center gap-[0.35rem] min-w-0 max-w-full";

/// 选项标题（模型面板下两行截断；选中态加粗）。
pub(crate) const CHAT_SHEET_TITLE: &str = "block min-w-0 max-w-full text-[0.78rem] font-semibold tracking-[-0.03em] text-secondary-3 group-data-[active=true]/opt:font-extrabold group-data-[active=true]/opt:text-secondary-2 group-data-[model=true]/mw:line-clamp-2 group-data-[model=true]/mw:overflow-hidden group-data-[model=true]/mw:[overflow-wrap:anywhere] group-data-[model=true]/mw:[word-break:break-word] group-data-[model=true]/mw:group-data-[active=true]/opt:font-[650] group-data-[model=true]/mw:group-data-[active=true]/opt:text-secondary-1";

/// 网络状态点（`data-ok` / `data-bad`）。
pub(crate) const CHAT_SHEET_NET_DOT: &str = "flex-none w-[0.45rem] h-[0.45rem] rounded-full shadow-[0_0_0_1px_color-mix(in_srgb,var(--primary-color)_55%,transparent)] data-[ok=true]:bg-[#22c55e] data-[bad=true]:bg-[#ef4444]";

pub(crate) const CHAT_SHEET_SUB: &str = "block text-[0.65rem] font-semibold text-secondary-3 leading-[1.35] group-data-[active=true]/opt:text-secondary-1";

pub(crate) const CHAT_SHEET_CHECK: &str = "absolute right-[0.55rem] top-1/2 -translate-y-1/2 text-[var(--ac-color-312e81)] pointer-events-none group-data-[custom=true]/row:right-[2.15rem]";

pub(crate) const CHAT_SHEET_EDIT: &str = "absolute right-[0.35rem] top-1/2 -translate-y-1/2 z-[1] inline-flex items-center justify-center w-[1.55rem] h-[1.55rem] m-0 p-0 border-0 rounded-[0.35rem] bg-transparent text-secondary-3 cursor-pointer hover:text-secondary-1 hover:bg-primary-2";

pub(crate) const CHAT_SHEET_EDIT_PANEL: &str = "flex-[1_1_100%] flex flex-col gap-[0.32rem] box-border pt-0 pr-[0.35rem] pb-[0.4rem] pl-[0.35rem] [&_.sheet-field]:w-full [&_.sheet-field]:flex-none";

pub(crate) const CHAT_SHEET_EDIT_ACTIONS: &str = "flex justify-end gap-[0.35rem]";

pub(crate) const CHAT_SHEET_EDIT_CANCEL: &str = "h-[1.7rem] m-0 px-[0.7rem] py-0 rounded-[0.4rem] text-[0.68rem] font-[650] cursor-pointer border border-primary-2 bg-transparent text-secondary-2";

pub(crate) const CHAT_SHEET_EDIT_SAVE: &str = "h-[1.7rem] m-0 px-[0.7rem] py-0 rounded-[0.4rem] text-[0.68rem] font-[650] cursor-pointer border border-[var(--border-color-dark)] bg-primary-1 text-secondary-1";

pub(crate) const CHAT_SHEET_CUSTOM: &str = "flex flex-wrap items-center gap-[0.35rem] mt-[0.35rem] pt-[0.3rem] pr-[0.25rem] pb-[0.15rem] pl-[0.25rem] border-t border-primary-2";

pub(crate) const CHAT_SHEET_CUSTOM_KEY: &str = "sheet-field box-border min-w-0 flex-[1_1_8rem] h-[1.85rem] m-0 px-[0.4rem] py-0 border rounded-[0.4rem] border-primary-2 bg-primary text-secondary-1 text-[0.65rem] font-semibold outline-none";

pub(crate) const CHAT_SHEET_CUSTOM_INPUT: &str = "sheet-field flex-[1_1_6.5rem] min-w-0 box-border h-[1.85rem] m-0 px-[0.55rem] py-0 border rounded-[0.4rem] border-primary-2 bg-primary text-secondary-1 text-[0.72rem] font-semibold outline-none focus:border-[var(--border-color-dark)]";

pub(crate) const CHAT_SHEET_CUSTOM_ACTIONS: &str = "inline-flex items-center justify-end gap-[0.35rem] w-full";

pub(crate) const CHAT_SHEET_CUSTOM_ICON_BTN: &str = "inline-flex items-center justify-center w-[2rem] h-[2rem] m-0 p-0 border rounded-[0.4rem] border-primary-2 bg-primary-1 text-secondary-1 cursor-pointer not-disabled:hover:bg-primary-2 disabled:opacity-72 disabled:cursor-wait data-[busy=true]:opacity-72 data-[busy=true]:cursor-wait";

pub(crate) const CHAT_SHEET_SPIN: &str = "animate-[ac-chat-mode-sheet-spin_0.85s_linear_infinite] motion-reduce:animate-none";

/// 上传附件按钮。
pub(crate) const CHAT_UPLOAD_BTN: &str = "relative box-border appearance-none inline-flex items-center justify-center w-[2rem] h-[2rem] shrink-0 text-secondary-1 hover:text-[var(--border-color-dark)]";

/// 隐藏的文件选择框。
pub(crate) const CHAT_FILE_INPUT: &str = "absolute inset-0 w-full h-full opacity-0 cursor-pointer text-[0px]";

// ——————————————————————————— 侧栏技能区（P5-3） ———————————————————————————

pub(crate) const SKILLS: &str = "flex flex-col gap-[0.65rem] min-h-0 flex-1 mt-[-0.35rem] mr-[-0.15rem] mb-0 ml-[-0.15rem]";

pub(crate) const SKILLS_SEARCH_ICON: &str = "shrink-0 text-secondary-3 opacity-75";

/// HOT 角标（侧栏切换钮内，取更紧凑的侧栏档尺寸）。
pub(crate) const SKILLS_HOT: &str = "rounded-full px-[0.28rem] py-[0.05rem] text-[0.55rem] font-black tracking-[0.02em] text-[color:var(--primary-color)] bg-[var(--secondary-error-color-light)]";

pub(crate) const SKILLS_STATIONS: &str = "flex items-center gap-[0.2rem] p-[0.12rem] rounded-[0.4rem] bg-[color-mix(in_srgb,var(--border-color-light)_55%,transparent)] shrink-0 min-h-[calc(1.5rem+0.24rem)] overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden";

pub(crate) const SKILLS_STATION_BTN: &str = "flex-[1_1_0] min-w-0 h-[1.5rem] px-[0.3rem] py-0 border-0 rounded-[0.3rem] inline-flex items-center justify-center bg-transparent text-secondary-3 text-[0.64rem] font-[750] whitespace-nowrap overflow-hidden text-ellipsis cursor-pointer data-[active=true]:text-secondary-1 data-[active=true]:bg-primary data-[active=true]:shadow-[0_1px_2px_var(--shadow-color)]";

pub(crate) const SKILLS_BANNER: &str = "px-[0.55rem] py-[0.45rem] rounded-[0.45rem] border border-border bg-[var(--ac-color-eff6ff)] text-[var(--ac-color-2563eb)] text-[0.68rem] font-[750] leading-[1.45] flex flex-col gap-[0.2rem] shrink-0 data-[error=true]:text-[var(--ac-color-991b1b)] data-[error=true]:border-[var(--ac-color-fecaca)] data-[error=true]:bg-[color-mix(in_srgb,var(--ac-color-fecaca)_55%,var(--primary-color))] data-[warn=true]:text-[var(--ac-color-92400e)] data-[warn=true]:border-[var(--ac-color-fde68a)] data-[warn=true]:bg-[var(--ac-color-fffbeb)]";

pub(crate) const SKILLS_HEADER: &str = "flex items-center justify-between gap-[0.5rem] pt-[0.15rem] px-[0.35rem] pb-0 text-secondary-3 text-[0.68rem] font-extrabold tracking-[0.06em] uppercase shrink-0";

pub(crate) const SKILLS_ICON: &str = "w-[1.85rem] h-[1.85rem] rounded-[0.4rem] inline-flex items-center justify-center shrink-0 mt-[0.05rem]";

pub(crate) const SKILLS_BODY: &str = "min-w-0 flex-1 flex flex-col gap-[0.12rem]";

pub(crate) const SKILLS_TITLE: &str = "text-secondary-1 text-[0.8rem] font-extrabold leading-[1.25] whitespace-nowrap overflow-hidden text-ellipsis";

pub(crate) const SKILLS_DESC: &str = "text-secondary-3 text-[0.7rem] font-semibold leading-[1.35] line-clamp-2 overflow-hidden";

pub(crate) const SKILLS_META: &str = "text-[color-mix(in_srgb,var(--secondary-color-3)_88%,var(--primary-color-5))] text-[0.62rem] font-[650] whitespace-nowrap overflow-hidden text-ellipsis";

pub(crate) const SKILLS_ACTIONS: &str = "flex items-center gap-[0.15rem] shrink-0 mt-[0.05rem]";

/// 技能行操作钮（`data-on` 已装备 / `data-danger` 危险操作）。
pub(crate) const SKILLS_ACTION: &str = "w-[1.55rem] h-[1.55rem] p-0 border-0 rounded-[0.35rem] inline-flex items-center justify-center text-[var(--skills-list-action-equip)] bg-transparent cursor-pointer transition-[color,opacity] duration-[0.12s] ease-out not-disabled:hover:text-[var(--skills-list-action-equip-on)] not-disabled:hover:opacity-85 data-[on=true]:text-[var(--skills-list-action-equip-on)] data-[danger=true]:text-[var(--skills-list-action-delete)] data-[danger=true]:hover:not-disabled:text-[color-mix(in_srgb,var(--skills-list-action-delete)_82%,var(--secondary-color-1))] disabled:opacity-45 disabled:cursor-not-allowed";

pub(crate) const SKILLS_BADGE: &str = "px-[0.35rem] py-[0.12rem] rounded-full bg-[var(--ac-color-eff6ff)] text-[var(--ac-color-2563eb)] text-[0.58rem] font-[850] whitespace-nowrap data-[muted=true]:bg-primary-1 data-[muted=true]:text-secondary-3";

pub(crate) const SKILLS_STATE: &str = "mt-[-0.9rem] mb-[1.1rem] mx-0 px-[0.95rem] py-[0.72rem] border rounded-[0.9rem] border-border bg-[var(--ac-color-eff6ff)] text-[var(--ac-color-2563eb)] text-[0.78rem] font-extrabold data-[error=true]:text-[var(--ac-color-991b1b)] data-[error=true]:border-[var(--ac-color-fecaca)] data-[error=true]:bg-[var(--ac-color-fecaca)]";

pub(crate) const SKILLS_DETAIL_PANE: &str = "w-full max-w-none m-0 p-0 min-h-full box-border";

pub(crate) const SKILLS_DETAIL_EMPTY: &str = "flex flex-col items-center justify-center text-center gap-[0.65rem] min-h-[16rem] px-[1.25rem] py-[2rem] text-secondary-3";

pub(crate) const SKILLS_DETAIL_EMPTY_H3: &str = "m-0 text-[1.05rem] font-extrabold text-secondary-3";

pub(crate) const SKILLS_DETAIL_EMPTY_P: &str = "m-0 max-w-[22rem] text-[0.86rem] leading-[1.6] font-[650]";

pub(crate) const SKILLS_DETAIL: &str = "flex flex-col gap-[1rem] px-[1.1rem] pt-[1.1rem] pb-[1.4rem] border-0 rounded-none bg-transparent";

pub(crate) const SKILLS_DETAIL_HEAD: &str = "flex items-start gap-[0.9rem]";

pub(crate) const SKILLS_DETAIL_ICON: &str = "w-[3rem] h-[3rem] rounded-[0.75rem] inline-flex items-center justify-center shrink-0";

pub(crate) const SKILLS_DETAIL_TITLES: &str = "min-w-0 flex flex-col gap-[0.25rem]";

pub(crate) const SKILLS_DETAIL_H2: &str = "m-0 text-secondary-1 text-[1.2rem] font-[850] tracking-[-0.02em] leading-[1.25]";

pub(crate) const SKILLS_DETAIL_SLUG: &str = "m-0 text-secondary-3 text-[0.78rem] font-[650] leading-[1.35] [overflow-wrap:anywhere]";

pub(crate) const SKILLS_DETAIL_DESC: &str = "m-0 text-secondary-3 text-[0.9rem] leading-[1.65] font-[650]";

pub(crate) const SKILLS_DETAIL_ACTIONS: &str = "flex flex-wrap items-center gap-[0.55rem] pt-[0.35rem]";

// ——————————————————————————— 会话历史侧栏（P5-3） ———————————————————————————

/// 历史面板（`group/hp` + `data-rail`；宽度由 inline style 同步）。
pub(crate) const CHAT_HISTORY_PANEL: &str = "group/hp relative w-[calc(2.25rem+1px)] max-w-[calc(2.25rem+1px)] min-w-0 h-full min-h-0 overflow-y-auto overflow-x-hidden shrink-0 self-stretch box-border py-[0.5rem] px-0 border-0 border-l border-primary-2 rounded-none bg-primary shadow-none translate-x-0 opacity-100 pointer-events-auto transition-[width,max-width,border-color,opacity,transform] duration-[0.22s] ease-out group-data-[resizing=true]/root:[transition:none]!";

pub(crate) const CHAT_HISTORY_RESIZER: &str = "absolute top-0 bottom-0 -left-[5px] w-[8px] cursor-col-resize z-[6] bg-transparent transition-[background] duration-150 ease-out hover:bg-[linear-gradient(90deg,var(--ac-color-4285f42e),var(--ac-color-4285f42e)_60%,transparent)]";

pub(crate) const CHAT_HISTORY_ERROR: &str = "mt-[0.65rem] text-[var(--ac-color-991b1b)] text-[0.72rem] leading-[1.45]";

/// 历史条目图标（hover / 选中 / 运行中变品牌色）。
pub(crate) const CHAT_HISTORY_RAIL_ICON: &str = "inline-flex items-center justify-center w-[1.75rem] h-[1.75rem] shrink-0 border-0 rounded-none text-secondary-3 box-border group-hover/item:text-brand group-data-[active=true]/item:text-brand group-data-[running=true]/item:text-brand";

/// 扁平新建按钮。
pub(crate) const CHAT_HISTORY_CREATE: &str = "w-full flex items-center justify-start gap-[0.25rem] box-border mb-[0.25rem] min-h-[2rem] px-[0.25rem] py-[0.3rem] border-0 rounded-none text-secondary-3 bg-transparent text-[0.75rem] font-bold text-center cursor-pointer overflow-hidden transition-[background,color] duration-150 ease-out hover:bg-primary-1 hover:text-brand disabled:opacity-50 disabled:cursor-not-allowed group-data-[rail=true]/hp:gap-0";

pub(crate) const CHAT_HISTORY_CREATE_ICON: &str = "inline-flex items-center justify-center w-[1.75rem] h-[1.75rem] shrink-0 border-0 rounded-none box-border text-current";

/// 历史行（`group/hrow` + `data-editing`）。
pub(crate) const CHAT_HISTORY_ROW: &str = "group/hrow block min-w-0 overflow-hidden [&+&]:mt-[0.1rem] data-[editing=true]:w-full data-[editing=true]:box-border data-[editing=true]:px-[0.25rem] data-[editing=true]:py-[0.25rem] data-[editing=true]:border-0 data-[editing=true]:rounded-none data-[editing=true]:bg-primary-2 data-[editing=true]:shadow-none";

pub(crate) const CHAT_HISTORY_EDIT_WRAP: &str = "relative w-full min-w-0 box-border overflow-hidden group-data-[rail=true]/hp:hidden!";

pub(crate) const CHAT_HISTORY_EDIT_ACTIONS: &str = "absolute top-1/2 right-[0.2rem] z-[1] flex items-center gap-[0.08rem] -translate-y-1/2 pointer-events-auto group-data-[rail=true]/hp:hidden!";

/// 历史条目（`group/item` + `data-active` / `data-running`）。
pub(crate) const CHAT_HISTORY_ITEM: &str = "group/item min-w-0 w-full flex items-center gap-[0.25rem] box-border px-[0.25rem] py-[0.35rem] border-0 rounded-none bg-transparent text-left cursor-pointer shadow-none overflow-hidden [contain:paint] transition-[background,color] duration-150 ease-out hover:bg-primary-1 group-data-[rail=true]/hp:justify-start group-data-[rail=true]/hp:gap-0 data-[active=true]:bg-[color-mix(in_srgb,var(--primary-color-2)_38%,var(--primary-color))] data-[active=true]:hover:bg-[color-mix(in_srgb,var(--primary-color-2)_38%,var(--primary-color))]";

pub(crate) const CHAT_HISTORY_ITEM_BODY: &str = "min-w-0 flex-[1_1_0] max-w-full flex flex-row items-center gap-0 overflow-hidden group-data-[rail=true]/hp:hidden!";

pub(crate) const CHAT_HISTORY_ITEM_PREVIEW_ROW: &str = "flex flex-row items-center gap-[0.2rem] min-w-0 flex-[1_1_0] overflow-hidden group-data-[rail=true]/hp:hidden!";

pub(crate) const CHAT_HISTORY_ITEM_PREVIEW: &str = "flex-[1_1_0] min-w-0 max-w-full m-0 p-0 border-0 bg-transparent overflow-hidden text-ellipsis whitespace-nowrap text-secondary-3 [font:inherit] text-[0.78rem] font-[550] leading-[1.35] text-left cursor-[inherit] pointer-events-none select-none group-data-[rail=true]/hp:hidden! group-data-[active=true]/item:text-secondary-1";

pub(crate) const CHAT_HISTORY_ITEM_RUNNING: &str = "shrink-0 ml-[6px] text-[11px] leading-[1.2] text-brand";

pub(crate) const CHAT_HISTORY_EDIT_ACTION: &str = "w-[1.35rem] min-w-[1.35rem] h-[1.35rem] min-h-[1.35rem] inline-flex items-center justify-center shrink-0 border-0 rounded-[0.3rem] bg-transparent text-secondary-3 cursor-pointer opacity-0 pointer-events-none transition-[opacity,color,background] duration-[0.12s] ease-out hover:text-brand hover:bg-primary-1 disabled:opacity-50 disabled:cursor-not-allowed group-data-[editing=true]/hrow:opacity-100 group-data-[editing=true]/hrow:pointer-events-auto";

pub(crate) const CHAT_HISTORY_TITLE_INPUT: &str = "min-w-0 w-full max-w-full h-[1.85rem] box-border border-0 rounded-[0.35rem] px-[0.45rem] py-0 text-secondary-1 bg-primary [font:inherit] text-[0.78rem] font-[650] outline-none focus:shadow-[0_0_0_2px_var(--primary-color-2)]";

// ——————————————————————————— Agent 运行详情弹窗（P5-3） ———————————————————————————

pub(crate) const AGENT_DETAIL_BACKDROP: &str = "fixed inset-0 z-[140] flex items-center justify-center p-[1.2rem] bg-[var(--shadow-color)] backdrop-blur-[10px]";

pub(crate) const AGENT_DETAIL_MODAL: &str = "w-[min(860px,94vw)] max-h-[min(82vh,760px)] flex flex-col overflow-hidden border rounded-[0.8rem] border-border-light bg-primary shadow-[0_28px_80px_var(--shadow-color)]";

pub(crate) const AGENT_DETAIL_HEAD: &str = "flex items-start justify-between gap-[1rem] px-[1.1rem] py-[1rem] border-b border-primary-2";

pub(crate) const AGENT_DETAIL_TITLE: &str = "m-0 text-secondary-1 text-[1rem] font-black";

pub(crate) const AGENT_DETAIL_SUBTITLE: &str = "mt-[0.22rem] mb-0 ml-0 mr-0 text-secondary-3 text-[0.76rem] leading-[1.45]";

pub(crate) const AGENT_DETAIL_CLOSE: &str = "w-[2rem] h-[2rem] inline-flex items-center justify-center shrink-0 border rounded-[0.55rem] border-border-light text-secondary-3 bg-primary cursor-default hover:text-brand hover:bg-[var(--ac-color-eff6ff)]";

/// 元信息徽章组（span 由 `[&>span]` 排）。
pub(crate) const AGENT_DETAIL_META: &str = "flex flex-wrap gap-[0.45rem] pt-[0.8rem] px-[1.1rem] pb-0 [&>span]:px-[0.55rem] [&>span]:py-[0.22rem] [&>span]:rounded-full [&>span]:text-[var(--ac-color-2563eb)] [&>span]:bg-[var(--ac-color-eff6ff)] [&>span]:text-[0.68rem] [&>span]:font-black";

pub(crate) const AGENT_DETAIL_COPY: &str = "ml-auto inline-flex items-center gap-[0.35rem] px-[0.58rem] py-[0.28rem] rounded-full border border-border text-[var(--ac-color-2563eb)] bg-[var(--ac-color-eff6ff)] text-[0.68rem] font-black cursor-pointer hover:bg-[var(--border-color-light)]";

pub(crate) const AGENT_DETAIL_BODY: &str = "grid gap-[0.85rem] overflow-y-auto px-[1.1rem] pt-[1rem] pb-[1.1rem]";

pub(crate) const AGENT_DETAIL_STEP: &str = "p-[0.9rem] border rounded-[0.7rem] border-primary-2 bg-primary-1";

/// 步骤/工具头（span 尾注由 `[&>span]` 排）。
pub(crate) const AGENT_DETAIL_STEP_HEAD: &str = "flex items-center justify-between gap-[0.75rem] text-secondary-1 text-[0.78rem] [&>span]:shrink-0 [&>span]:text-secondary-3 [&>span]:text-[0.68rem] [&>span]:font-[850]";

pub(crate) const AGENT_DETAIL_TIMING: &str = "flex flex-wrap gap-[0.4rem] mt-[0.45rem] [&>span]:px-[0.5rem] [&>span]:py-[0.18rem] [&>span]:rounded-[0.4rem] [&>span]:text-secondary-2 [&>span]:bg-primary-2 [&>span]:text-[0.66rem] [&>span]:font-extrabold";

pub(crate) const AGENT_DETAIL_TOOL: &str = "mt-[0.8rem] pt-[0.8rem] border-t border-primary-2";

pub(crate) const AGENT_DETAIL_BLOCK_TITLE: &str = "mt-[0.7rem] mb-[0.35rem] mx-0 text-secondary-3 text-[0.68rem] font-black uppercase";

pub(crate) const AGENT_DETAIL_PRE: &str = "max-h-[16rem] m-0 overflow-auto whitespace-pre-wrap [overflow-wrap:anywhere] border rounded-[0.55rem] border-border-light p-[0.7rem] text-secondary-3 bg-primary text-[0.72rem] leading-[1.55] [font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,'Liberation_Mono',monospace]";

pub(crate) const AGENT_DETAIL_ERROR: &str = "p-[1.2rem] text-[var(--ac-color-991b1b)] text-[0.9rem] font-[750]";

pub(crate) const AGENT_DETAIL_SKEL_WRAP: &str = "px-0 py-[1rem]";

pub(crate) const AGENT_DETAIL_SKEL_META: &str = "flex flex-wrap gap-[0.55rem] mb-[1rem]";

pub(crate) const AGENT_DETAIL_SKEL_CHIP: &str = "w-[6rem] h-[1.1rem] rounded-full";

pub(crate) const AGENT_DETAIL_SKEL_BODY: &str = "flex flex-col gap-[0.55rem]";

pub(crate) const AGENT_DETAIL_SKEL_LINE: &str = "w-full h-[0.78rem] rounded-[0.32rem]";

pub(crate) const AGENT_DETAIL_SKEL_LINE_SHORT: &str = "w-[72%]";

// ——————————————————————————— 设置提示（P5-3） ———————————————————————————

pub(crate) const SETTINGS_SAVED: &str = "mt-[0.35rem] mb-[0.75rem] mx-0 text-[0.72rem] font-semibold text-[var(--primary-error-color-light)] shrink-0 data-[ok=true]:text-[var(--ac-color-22c55e)]";

// ——————————————————————————— 原生标题栏（P5-4，desktop/chrome.rs） ———————————————————————————

pub(crate) const NATIVE_TITLEBAR: &str = "flex items-center h-[32px] shrink-0 px-[10px] py-0 gap-[8px] box-border select-none bg-[color-mix(in_srgb,var(--primary-color-1)_80%,transparent)] border-b border-[color-mix(in_srgb,var(--primary-color-2)_60%,transparent)]";

pub(crate) const NATIVE_TITLEBAR_WINDOWS: &str = "pt-0 pr-0 pb-0 pl-[10px] gap-[6px]";

pub(crate) const NATIVE_TITLEBAR_MACOS: &str = "h-[28px] pt-0 pr-[8px] pb-0 pl-0 gap-[6px] ac-native-titlebar--macos";

pub(crate) const NATIVE_TITLEBAR_DRAG: &str = "flex-1 flex items-center self-stretch min-h-0 cursor-default text-[13px] text-secondary-3 font-semibold";

pub(crate) const NATIVE_TITLEBAR_ACTIONS: &str = "inline-flex items-center justify-center self-center gap-[2px]";

pub(crate) const NATIVE_TITLEBAR_BTN: &str = "appearance-none border-0 shadow-none bg-transparent box-border inline-flex items-center justify-center w-[24px] h-[24px] min-w-[24px] min-h-[24px] p-0 m-0 leading-none rounded-[5px] hover:shadow-none focus-visible:outline-2 focus-visible:outline-[color-mix(in_srgb,var(--brand)_55%,transparent)] focus-visible:outline-offset-2 [&_svg]:block [&_svg]:shrink-0";

pub(crate) const NATIVE_TRAFFIC_SPACER: &str = "shrink-0 w-[76px] h-[14px] pointer-events-none";

pub(crate) const NATIVE_CAPTION: &str = "flex flex-row items-stretch shrink-0 h-full ml-[4px]";

pub(crate) const NATIVE_CAPTION_BTN: &str = "appearance-none relative inline-flex items-center justify-center w-[46px] h-full min-w-[46px] m-0 p-0 border-0 rounded-none bg-transparent text-secondary-1 cursor-default shadow-none hover:bg-[color-mix(in_srgb,var(--secondary-color-1)_8%,transparent)] active:bg-[color-mix(in_srgb,var(--secondary-color-1)_14%,transparent)] focus-visible:outline-2 focus-visible:outline-[color-mix(in_srgb,var(--brand)_55%,transparent)] focus-visible:outline-offset-[-2px]";

pub(crate) const NATIVE_CAPTION_CLOSE: &str = "hover:bg-[#e81123] hover:text-white active:bg-[#f1707a] active:text-white";

pub(crate) const NATIVE_CAPTION_ICON: &str = "block w-[10px] h-[10px] box-border pointer-events-none";

pub(crate) const NATIVE_CAPTION_ICON_MIN: &str = "w-[10px] h-px bg-current";

pub(crate) const NATIVE_CAPTION_ICON_MAX: &str = "w-[10px] h-[10px] border border-current bg-transparent";

pub(crate) const NATIVE_CAPTION_ICON_RESTORE: &str = "relative w-[10px] h-[10px] border-0 bg-transparent before:[content:''] before:absolute before:box-border before:border before:border-current before:bg-transparent before:top-0 before:right-0 before:w-[8px] before:h-[8px] after:[content:''] after:absolute after:left-0 after:bottom-0 after:w-[8px] after:h-[8px] after:border after:border-current after:bg-[color-mix(in_srgb,var(--primary-color-1)_80%,transparent)]";

pub(crate) const NATIVE_CAPTION_ICON_CLOSE: &str = "relative w-[10px] h-[10px] before:[content:''] before:absolute before:left-1/2 before:top-1/2 before:w-[12px] before:h-px before:bg-current before:-translate-x-1/2 before:-translate-y-1/2 before:rotate-45 after:[content:''] after:absolute after:left-1/2 after:top-1/2 after:w-[12px] after:h-px after:bg-current after:-translate-x-1/2 after:-translate-y-1/2 after:-rotate-45";

pub(crate) const NATIVE_RESIZE_HIT: &str = "fixed z-[100] pointer-events-auto";

pub(crate) const NATIVE_RESIZE_N: &str = "top-0 left-[8px] right-[8px] h-[8px] cursor-ns-resize";

pub(crate) const NATIVE_RESIZE_S: &str = "bottom-0 left-[8px] right-[8px] h-[8px] cursor-ns-resize";

pub(crate) const NATIVE_RESIZE_E: &str = "top-[8px] bottom-[8px] right-0 w-[8px] cursor-ew-resize";

pub(crate) const NATIVE_RESIZE_W: &str = "top-[8px] bottom-[8px] left-0 w-[8px] cursor-ew-resize";

pub(crate) const NATIVE_RESIZE_NE: &str = "top-0 right-0 w-[12px] h-[12px] cursor-nesw-resize";

pub(crate) const NATIVE_RESIZE_NW: &str = "top-0 left-0 w-[12px] h-[12px] cursor-nwse-resize";

pub(crate) const NATIVE_RESIZE_SE: &str = "right-0 bottom-0 w-[12px] h-[12px] cursor-nwse-resize";

pub(crate) const NATIVE_RESIZE_SW: &str = "bottom-0 left-0 w-[12px] h-[12px] cursor-nesw-resize";

// ——————————————————————————— Web 标题栏（P5-4，console.rs / theme.rs） ———————————————————————————

/// 顶部恢复条（悬浮时可下探；`data-pinned` 常驻展开）。
pub(crate) const WEB_TITLEBAR_REVEAL: &str = "group/tbr fixed top-0 left-0 right-0 w-full h-[10px] z-[999] border-b border-primary-2 hover:h-[40px] data-[pinned=true]:h-[40px] before:[content:''] before:absolute before:top-0 before:left-0 before:w-[280px] before:h-[4px] before:bg-[linear-gradient(90deg,var(--ac-color-93c5fd99)_0%,transparent_100%)] before:transition-[opacity,height,background] before:duration-[0.18s] before:ease-in-out dark:border-b-transparent dark:after:[content:''] dark:after:absolute dark:after:left-0 dark:after:right-0 dark:after:bottom-0 dark:after:h-px dark:after:z-[6] dark:after:pointer-events-none dark:after:opacity-[var(--ac-sand-opacity)] dark:after:bg-[var(--ac-sand-grad-x)]";

/// 聊天栏内停靠形态。
pub(crate) const WEB_TITLEBAR_DOCK: &str = "relative! top-auto! left-auto! right-auto! w-full h-auto! min-h-[34px] z-[12] shrink-0 hover:h-auto! dark:before:hidden";

/// 顶层悬浮恢复条（覆盖在内容上方，仅自身可点）。
pub(crate) const WEB_TITLEBAR_FLOATING: &str = "absolute top-0 left-0 right-0 z-[90] pointer-events-none [&>*]:pointer-events-auto";

/// 标题栏内容行。
pub(crate) const WEB_TITLEBAR: &str = "absolute top-0 left-0 right-0 flex items-center h-[34px] px-4 py-0 group-data-[dock=true]/tbr:relative group-data-[dock=true]/tbr:inset-auto group-data-[dock=true]/tbr:h-[34px]";

pub(crate) const WEB_TITLEBAR_ACTIONS: &str = "flex items-center gap-[6px]";

pub(crate) const WEB_TITLEBAR_ACTIONS_END: &str = "flex items-center gap-[6px]";

pub(crate) const WEB_TITLEBAR_SPACER: &str = "flex-1 min-w-[8px]";

/// 切换按钮（`data-menu-open` 下拉展开时同 hover 底）。
pub(crate) const WEB_TITLEBAR_BTN: &str = "h-[20px] px-[6px] py-0 border-0 rounded-[4px] text-brand text-[12px] font-extrabold leading-none bg-transparent cursor-default transition-[background,color] duration-150 hover:bg-[var(--border-color-light)] data-[menu-open=true]:bg-[var(--border-color-light)]";

pub(crate) const WEB_TITLEBAR_ICON: &str = "block shrink-0";

pub(crate) const WEB_TITLEBAR_SWITCH_WRAP: &str = "inline-flex items-center gap-0";

pub(crate) const WEB_TITLEBAR_SWITCH_ICON_OUTSIDE: &str = "min-w-[28px] h-[20px] px-[6px] py-0 border-0 rounded-[4px] inline-flex items-center justify-center text-[var(--brand-light)] flex-[0_0_auto] bg-transparent cursor-pointer hover:bg-[var(--border-color-light)]";

pub(crate) const WEB_TITLEBAR_SETTINGS_WRAP: &str = "relative flex items-center";

pub(crate) const WEB_TITLEBAR_MENU_BACKDROP: &str = "fixed inset-0 z-[1180] bg-transparent";

/// 设置下拉菜单（暗色为银河描边弹层）。
pub(crate) const WEB_TITLEBAR_MENU: &str = "absolute top-[calc(100%+4px)] right-0 z-[1181] min-w-[11.5rem] p-[0.35rem] rounded-[0.55rem] border border-primary-2 bg-primary shadow-[0_2px_6px_var(--shadow-color),0_14px_36px_-6px_var(--shadow-color)] dark:[border:1px_solid_transparent] dark:[background-origin:border-box] dark:[background-clip:padding-box,border-box] dark:bg-[linear-gradient(var(--primary-color),var(--primary-color)),var(--ac-sand-grad-border)] dark:bg-primary dark:shadow-[0_12px_36px_var(--shadow-color),inset_0_1px_0_rgb(255_255_255/0.05)]";

pub(crate) const WEB_TITLEBAR_MENU_ITEM: &str = "flex items-center justify-between gap-[0.65rem] w-full box-border m-0 px-[0.55rem] py-[0.45rem] border-0 rounded-[0.4rem] bg-transparent [font:inherit] text-[0.8rem] font-[650] text-secondary-3 text-left cursor-pointer whitespace-nowrap hover:bg-primary-1 data-[active=true]:text-[var(--ac-color-2563eb)]";

// ——————————————————————————— 终端面板（P5-4，desktop/terminal.rs） ———————————————————————————

pub(crate) const TERMINAL_PANEL_HOST: &str = "flex-[0_0_auto] flex flex-col min-h-0";

/// 终端面板（暗色顶部银河横线；等宽字体族内联）。
pub(crate) const TERMINAL_PANEL: &str = "relative flex-[1_1_auto] flex flex-col min-h-0 h-full border-t border-primary-2 bg-primary-1 dark:border-t-transparent dark:before:[content:''] dark:before:absolute dark:before:left-0 dark:before:right-0 dark:before:top-0 dark:before:h-px dark:before:z-[6] dark:before:pointer-events-none dark:before:opacity-[var(--ac-sand-opacity)] dark:before:bg-[var(--ac-sand-grad-x)]";

pub(crate) const TERMINAL_RESIZER: &str = "absolute -top-[3px] left-0 right-0 h-[6px] cursor-row-resize z-[2] bg-transparent";

pub(crate) const TERMINAL_TOOLBAR: &str = "flex items-center justify-between gap-[8px] shrink-0 h-[1.75rem] pt-0 pr-[0.2rem] pb-0 pl-[10px] box-border bg-transparent";

pub(crate) const TERMINAL_TOOLBAR_TITLE: &str = "text-[11px] font-medium text-secondary-3 tracking-[0.01em]";

pub(crate) const TERMINAL_TOOLBAR_ACTIONS: &str = "inline-flex items-center gap-[2px]";

pub(crate) const TERMINAL_ADD_PUSA_BTN: &str = "appearance-none border border-[color-mix(in_srgb,var(--primary-color-2)_80%,transparent)] bg-[color-mix(in_srgb,var(--primary-color-2)_35%,transparent)] text-secondary-1 h-[1.375rem] px-[0.55rem] py-0 mr-[0.2rem] rounded-[0.35rem] text-[0.68rem] font-[650] tracking-[0.01em] leading-none cursor-pointer whitespace-nowrap hover:bg-[color-mix(in_srgb,var(--primary-color-2)_55%,transparent)] hover:border-[color-mix(in_srgb,var(--border-color-hover)_50%,var(--primary-color-2))]";

pub(crate) const TERMINAL_TOOLBAR_BTN: &str = "appearance-none border-0 bg-transparent text-[var(--text-tertiary)] w-[1.375rem] h-[1.375rem] inline-flex items-center justify-center rounded-[4px] cursor-pointer p-0 flex-[0_0_1.375rem] hover:bg-[color-mix(in_srgb,var(--primary-color-2)_45%,transparent)] hover:text-secondary-1";

pub(crate) const TERMINAL_TOOLBAR_ADD: &str = "text-secondary-2";

pub(crate) const TERMINAL_MAIN: &str = "flex-[1_1_auto] min-h-0 flex flex-row items-stretch";

/// xterm 容器（留白放外层避免末行裁切）。
pub(crate) const TERMINAL_BODY_XTERM: &str = "flex-[1_1_auto] min-w-0 min-h-0 overflow-hidden flex flex-col items-stretch justify-start [direction:ltr] text-left p-0 box-border pt-[4px] pr-0 pb-[6px] pl-[8px] bg-white dark:bg-[#0e0e10]";

/// xterm 挂载点（类名保留供 JS 桥）。
pub(crate) const TERMINAL_XTERM_HOST: &str = "ac-xterm-host flex-[1_1_auto] min-w-0 min-h-0 w-full h-full overflow-hidden bg-white dark:bg-[#0e0e10]";

pub(crate) const TERMINAL_SESSION_RAIL: &str = "flex-[0_0_1.75rem] w-[1.75rem] box-border flex flex-col items-center gap-[2px] pt-[0.2rem] pr-0 pb-[8px] pl-0 border-l border-[color-mix(in_srgb,var(--primary-color-2)_70%,transparent)] bg-transparent";

/// 会话圆钮（`data-active` 选中）。
pub(crate) const TERMINAL_SESSION_BTN: &str = "appearance-none border-0 bg-transparent text-[var(--text-tertiary)] w-[1.375rem] h-[1.375rem] inline-flex items-center justify-center rounded-[4px] cursor-pointer p-0 flex-[0_0_1.375rem] hover:bg-[color-mix(in_srgb,var(--primary-color-2)_55%,transparent)] hover:text-secondary-1 data-[active=true]:bg-[color-mix(in_srgb,var(--primary-color-2)_80%,transparent)] data-[active=true]:text-secondary-1";

pub(crate) const TERMINAL_SESSION_CLOSE: &str = "mt-auto";

/// 终端错误块（`data-err` 红色）。
pub(crate) const TERMINAL_ERROR: &str = "flex-[1_1_auto] flex flex-col items-start justify-center gap-[6px] px-[14px] py-[16px] text-[12px] leading-[1.5] text-secondary-2 [font-family:'SF_Mono',Menlo,Monaco,ui-monospace,'Cascadia_Mono',Consolas,monospace] data-[err=true]:text-[var(--ac-color-50140fa6,#b91c1c)]";

// ——————————————————————————— 中间栏文本编辑器 ———————————————————————————

/// 编辑器根容器（字号/行高用自定义属性，gutter 与文本层共用）。
pub(crate) const EDITOR: &str = "relative flex flex-col gap-0 h-full min-h-0 [--ac-editor-fs:0.8rem] [--ac-editor-lh:1.45]";

/// 查找 / 替换浮层。
pub(crate) const FIND: &str = "absolute top-[0.55rem] right-3 z-[8] flex flex-col gap-[0.35rem] min-w-[min(420px,calc(100%-1.5rem))] max-w-[min(520px,calc(100%-1.5rem))] px-[0.45rem] py-[0.4rem] rounded-lg border border-[color-mix(in_srgb,var(--secondary-color-3)_28%,transparent)] bg-[color-mix(in_srgb,var(--primary-color,#fff)_92%,var(--secondary-color-3))] shadow-[0_10px_28px_rgb(15_23_42/0.14)] box-border dark:border-[#2e2e33] dark:bg-[#1c1c20] dark:shadow-[0_12px_32px_rgb(0_0_0/0.45)]";

/// 浮层里的一行。
pub(crate) const FIND_ROW: &str = "flex items-center gap-[0.35rem]";

/// 展开替换行的箭头钮。
pub(crate) const FIND_TOGGLE: &str = "shrink-0 inline-flex items-center justify-center w-[1.6rem] h-[1.75rem] mt-[0.05rem] border-0 rounded-md bg-transparent text-[var(--text-secondary,var(--secondary-color-3))] cursor-pointer hover:bg-[color-mix(in_srgb,var(--secondary-color-3)_12%,transparent)] hover:text-secondary-1 dark:text-[#a1a1aa] dark:hover:bg-[#2a2a2e] dark:hover:text-[#f4f4f5]";

/// 占位（与箭头同宽，保持下一行输入框对齐）。
pub(crate) const FIND_SPACER: &str = "shrink-0 inline-flex items-center justify-center w-[1.6rem] h-[1.75rem] mt-[0.05rem] border-0 rounded-md bg-transparent pointer-events-none invisible";

/// 输入框外壳（聚焦时蓝色描边 + 光圈）。
pub(crate) const FIND_INPUT_WRAP: &str = "flex-auto min-w-0 flex items-center gap-[0.15rem] min-h-[1.85rem] pl-[0.45rem] pr-[0.2rem] rounded-md border border-[color-mix(in_srgb,var(--secondary-color-3)_26%,transparent)] bg-[var(--primary-color,#fff)] box-border focus-within:border-[color-mix(in_srgb,var(--ac-color-3b82f6,#3b82f6)_70%,transparent)] focus-within:shadow-[0_0_0_2px_color-mix(in_srgb,var(--ac-color-3b82f6,#3b82f6)_22%,transparent)] dark:border-[#2e2e33] dark:bg-[#141416] dark:focus-within:border-[#3b82f6] dark:focus-within:shadow-[0_0_0_2px_rgb(59_130_246/0.28)]";

/// 查找 / 替换输入框。
pub(crate) const FIND_INPUT: &str = "flex-auto min-w-0 h-[1.7rem] border-0 outline-none bg-transparent text-secondary-1 text-[0.8rem] placeholder:text-[var(--text-secondary,var(--secondary-color-3))] placeholder:opacity-75 dark:text-[#f4f4f5] dark:placeholder:text-[#71717a]";

/// 选项开关（Aa / 全字 / 正则）；激活态交给 `data-active`。
pub(crate) const FIND_OPT: &str = "shrink-0 inline-flex items-center justify-center w-[1.45rem] h-[1.45rem] border-0 rounded bg-transparent text-[var(--text-secondary,var(--secondary-color-3))] cursor-pointer hover:bg-[color-mix(in_srgb,var(--secondary-color-3)_12%,transparent)] hover:text-secondary-1 data-[active=true]:bg-[color-mix(in_srgb,var(--ac-color-3b82f6,#3b82f6)_18%,transparent)] data-[active=true]:text-[var(--ac-color-1d4ed8,#1d4ed8)] dark:text-[#a1a1aa] dark:hover:bg-[#2a2a2e] dark:hover:text-[#f4f4f5] dark:data-[active=true]:bg-[rgb(59_130_246/0.22)] dark:data-[active=true]:text-[#93c5fd]";

/// 行尾按钮组容器。
pub(crate) const FIND_ACTIONS: &str = "shrink-0 inline-flex items-center justify-end gap-[0.1rem] min-w-[9.5rem] mt-[0.05rem]";

/// 命中计数；无命中时变橙（`data-empty`）。
pub(crate) const FIND_COUNT: &str = "min-w-[5.5rem] mr-[0.2rem] text-[0.72rem] font-semibold text-[var(--text-secondary,var(--secondary-color-3))] text-right whitespace-nowrap data-[empty=true]:text-[var(--ac-color-f97316,#f97316)] dark:text-[#a1a1aa] dark:data-[empty=true]:text-[#fb923c]";

/// 上一个 / 下一个 / 关闭 / 替换 图标钮。
pub(crate) const FIND_BTN: &str = "inline-flex items-center justify-center w-[1.55rem] h-[1.55rem] border-0 rounded-[5px] bg-transparent text-[var(--secondary-color-2,var(--secondary-color-3))] cursor-pointer enabled:hover:bg-[color-mix(in_srgb,var(--secondary-color-3)_12%,transparent)] enabled:hover:text-secondary-1 disabled:opacity-35 disabled:cursor-default dark:text-[#a1a1aa] dark:enabled:hover:bg-[#2a2a2e] dark:enabled:hover:text-[#f4f4f5]";

/// 编辑器工具栏（保存态 / 操作按钮）。
pub(crate) const EDITOR_TOOLBAR: &str =
    "flex items-center justify-start gap-3 shrink-0 px-[1.1rem] pt-[0.55rem] pb-[0.35rem]";

/// 工具栏按钮组。
pub(crate) const EDITOR_ACTIONS: &str = "inline-flex items-center gap-[0.45rem] shrink-0";

/// 「未保存」标记。
pub(crate) const EDITOR_DIRTY: &str = "text-[0.7rem] font-bold text-[var(--ac-color-f97316)]";

/// 「已保存」提示。
pub(crate) const EDITOR_SAVE_NOTICE: &str =
    "text-[0.7rem] font-[650] text-[var(--ac-color-047857)]";

/// 滚动区。
pub(crate) const EDITOR_SCROLL: &str = "flex-1 min-h-0 overflow-auto";

/// 代码行容器（行数由 `--ac-editor-lines` 控制）。
pub(crate) const EDITOR_CODE: &str = "flex items-stretch min-h-full box-border [--ac-editor-lines:1]";

/// 行号槽。暗色下用 `::after` 画一条「流沙渐变」替代右边框。
pub(crate) const EDITOR_GUTTER: &str = "shrink-0 sticky left-0 z-[2] min-w-[2.6rem] px-[0.55rem] pt-[0.85rem] pb-[1.1rem] pl-[0.65rem] box-border text-right select-none text-[var(--text-secondary,var(--secondary-color-3))] opacity-62 font-mono [font-size:var(--ac-editor-fs)] [line-height:var(--ac-editor-lh)] bg-[color-mix(in_srgb,var(--secondary-color-3)_7%,var(--primary-color,#fff))] border-r border-[color-mix(in_srgb,var(--secondary-color-3)_18%,transparent)] dark:bg-[#121214] dark:border-r-transparent dark:text-[#6e7681] dark:opacity-100 dark:after:[content:''] dark:after:absolute dark:after:top-0 dark:after:right-0 dark:after:bottom-0 dark:after:w-px dark:after:z-[2] dark:after:pointer-events-none dark:after:opacity-[var(--ac-sand-opacity)] dark:after:bg-[var(--ac-sand-grad-y)]";

/// 单行行号。
pub(crate) const EDITOR_LN: &str = "h-[calc(var(--ac-editor-fs)*var(--ac-editor-lh))]";

/// 文本层容器（查找底色 / 高亮 / textarea 三层叠加）。
pub(crate) const EDITOR_SURFACE: &str = "relative flex-1 min-w-0 w-full";

/// 查找命中底色层（透明字，只露 mark 背景）。
pub(crate) const EDITOR_FIND_MARKS: &str = "absolute inset-0 z-0 box-border m-0 px-[1.1rem] pt-[0.85rem] pb-[1.1rem] pl-[0.75rem] overflow-hidden pointer-events-none whitespace-pre [tab-size:4] [word-wrap:normal] [overflow-wrap:normal] text-transparent [-webkit-text-fill-color:transparent] font-mono [font-size:var(--ac-editor-fs)] [line-height:var(--ac-editor-lh)]";

/// 单条命中底色。
pub(crate) const FIND_HIT: &str = "text-transparent [-webkit-text-fill-color:transparent] bg-[color-mix(in_srgb,#eab308_42%,transparent)] rounded-[2px] [box-decoration-break:clone] [-webkit-box-decoration-break:clone] dark:bg-[color-mix(in_srgb,#eab308_38%,transparent)]";

/// 当前命中的底色（更亮 + 描边）。
pub(crate) const FIND_HIT_CURRENT: &str = "text-transparent [-webkit-text-fill-color:transparent] bg-[color-mix(in_srgb,#f59e0b_72%,transparent)] rounded-[2px] [box-decoration-break:clone] [-webkit-box-decoration-break:clone] outline outline-1 outline-[color-mix(in_srgb,#d97706_55%,transparent)] dark:bg-[color-mix(in_srgb,#fbbf24_55%,transparent)] dark:outline-[color-mix(in_srgb,#f59e0b_65%,transparent)]";

/// 语法高亮层。
pub(crate) const EDITOR_HIGHLIGHT: &str = "absolute inset-0 z-[1] box-border m-0 px-[1.1rem] pt-[0.85rem] pb-[1.1rem] pl-[0.75rem] overflow-hidden pointer-events-none whitespace-pre [tab-size:4] [word-wrap:normal] [overflow-wrap:normal] text-secondary-1 bg-transparent font-mono [font-size:var(--ac-editor-fs)] [line-height:var(--ac-editor-lh)] dark:text-[#cccccc]";

/// 真正的输入层（透明字，只留光标）。
pub(crate) const EDITOR_TEXTAREA: &str = "relative z-[2] block w-full min-w-0 box-border resize-none overflow-hidden h-[calc(var(--ac-editor-lines)*var(--ac-editor-fs)*var(--ac-editor-lh))] min-h-[calc(100%-1.95rem)] m-0 px-[1.1rem] pt-[0.85rem] pb-[1.1rem] pl-[0.75rem] border-0 rounded-none bg-transparent text-transparent [-webkit-text-fill-color:transparent] caret-secondary-1 font-mono [font-size:var(--ac-editor-fs)] [line-height:var(--ac-editor-lh)] whitespace-pre [tab-size:4] outline-none selection:bg-[color-mix(in_srgb,var(--brand,#3b82f6)_28%,transparent)] dark:[caret-color:#cccccc]";

/// 读取出错提示。
pub(crate) const EDITOR_ERROR: &str =
    "px-[1.1rem] py-5 text-center text-[0.85rem] font-[650] text-error";

/// 加载中提示。
pub(crate) const EDITOR_LOADING: &str =
    "px-[1.1rem] py-5 text-center text-[0.85rem] font-[650] text-secondary-3";

// ——————————————————————————— 中间栏 diff 视图 ———————————————————————————

/// diff 根容器。
pub(crate) const DIFF: &str =
    "flex flex-col gap-[0.65rem] h-full min-h-0 px-[0.85rem] pt-3 pb-[0.85rem] box-border overflow-hidden";

/// diff 头部行。
pub(crate) const DIFF_HEADER: &str =
    "flex items-start justify-between gap-3 shrink-0 px-1";

/// 标题区。
pub(crate) const DIFF_HEADING: &str = "min-w-0";

/// 主标题。
pub(crate) const DIFF_TITLE: &str = "m-0 text-base font-bold text-secondary-1";

/// 副标题。
pub(crate) const DIFF_SUBTITLE: &str =
    "mt-[0.2rem] mx-0 mb-0 text-[0.74rem] font-[550] text-[var(--text-secondary,var(--secondary-color-3))]";

/// 文件路径。
pub(crate) const DIFF_PATH: &str = "mt-1 mx-0 mb-0 overflow-hidden text-ellipsis whitespace-nowrap font-mono text-[0.7rem] text-[var(--text-secondary,var(--secondary-color-3))] opacity-80";

/// +/- 统计行。
pub(crate) const DIFF_STATS: &str =
    "inline-flex gap-[0.55rem] mt-[0.35rem] mx-0 mb-0 font-mono text-[0.72rem] font-bold";

/// 新增行数。
pub(crate) const DIFF_STAT_ADD: &str = "text-[var(--ac-color-047857)]";

/// 删除行数。
pub(crate) const DIFF_STAT_DEL: &str = "text-[var(--ac-color-991b1b)]";

/// 「打开文件」按钮。
pub(crate) const DIFF_OPEN_BTN: &str = "shrink-0 px-[0.7rem] py-[0.35rem] border border-[var(--ac-refresh-pill-border,#c9cfdb)] rounded-[0.4rem] bg-[var(--ac-refresh-pill-bg,#fff)] text-[var(--ac-refresh-pill-text,#5c6370)] text-[0.72rem] font-[650] cursor-pointer hover:border-[var(--ac-refresh-pill-hover-border,#a8b2c4)] hover:bg-[var(--ac-refresh-pill-hover-bg,#fafbfd)] hover:text-[var(--ac-refresh-pill-hover-text,#3d4552)]";

/// diff 正文外框。
pub(crate) const DIFF_INLINE: &str = "flex flex-col flex-auto min-h-0 border border-[color-mix(in_srgb,var(--secondary-color-3)_22%,transparent)] rounded-[0.45rem] overflow-hidden bg-[color-mix(in_srgb,var(--secondary-color-3)_4%,transparent)]";

/// diff 正文滚动区。
pub(crate) const DIFF_INLINE_BODY: &str =
    "m-0 px-0 py-1 flex-1 min-h-0 overflow-auto font-mono text-[0.74rem] leading-[1.45]";

/// 单行：`data-kind` = `add` / `del` / 空。
pub(crate) const DIFF_LINE: &str = "group grid grid-cols-[2.6rem_2.6rem_1.1rem_minmax(0,1fr)] gap-x-[0.35rem] items-start pl-0 pr-[0.55rem] min-h-[1.45em] whitespace-pre-wrap break-words text-secondary-1 data-[kind=del]:bg-[color-mix(in_srgb,var(--ac-color-fecaca)_48%,transparent)] data-[kind=del]:text-[var(--ac-color-991b1b)] data-[kind=add]:bg-[color-mix(in_srgb,var(--ac-color-bbf7d0)_52%,transparent)] data-[kind=add]:text-[var(--ac-color-047857)] dark:data-[kind=del]:text-[#f85149] dark:data-[kind=add]:text-[#3fb950]";

/// 行号。
pub(crate) const DIFF_LN: &str = "text-right select-none opacity-55 pt-[0.02em] text-[var(--text-secondary,var(--secondary-color-3))] group-data-[kind=del]:opacity-75 group-data-[kind=add]:opacity-75";

/// 左侧 +/- 标记。
pub(crate) const DIFF_MARK: &str = "shrink-0 text-center font-bold opacity-90 select-none";

/// 行内容（内部 `<span>` 为语法高亮 token，需要还原字重/斜体）。
pub(crate) const DIFF_TEXT: &str = "min-w-0 pr-[0.4rem] whitespace-pre font-mono [&_span]:whitespace-pre [&_span]:[font-style:inherit] [&_span]:[font-weight:inherit]";

// ——————————————————————————— 控制台三栏骨架 ———————————————————————————

/// 应用根容器（`overflow-hidden` 是常态；菜单/浮层打开时由 `:has()` 放开，见 app.css）。
pub(crate) const CONSOLE_ROOT: &str =
    "group/root relative flex flex-1 min-h-0 overflow-hidden text-secondary-1 data-[resizing=true]:cursor-col-resize data-[resizing=true]:select-none";

/// 侧栏外壳。过渡逐属性写在任意值里，保留原先 0.22s / 0.18s 的差异。
pub(crate) const SIDEBAR: &str = "relative min-w-0 w-[280px] flex-[0_0_280px] flex flex-col overflow-hidden border-r border-line bg-primary opacity-100 shadow-[2px_4px_20px_var(--shadow-color)] transition-[flex-basis_0.22s_ease,width_0.22s_ease,opacity_0.18s_ease,transform_0.22s_ease,border-color_0.18s_ease] dark:border-r-transparent dark:before:[content:''] dark:before:absolute dark:before:top-0 dark:before:right-0 dark:before:bottom-0 dark:before:w-px dark:before:z-[6] dark:before:pointer-events-none dark:before:opacity-[var(--ac-sand-opacity)] dark:before:bg-[var(--ac-sand-grad-y)] group-data-[resizing=true]/root:[transition:none]!";

/// 收起中：宽度插值到 0（1s 长缓动），仍然留在 DOM 里。
pub(crate) const SIDEBAR_SHUTTING: &str = "pointer-events-none will-change-[flex-basis,width,transform,opacity] min-w-0! max-w-0! w-0! flex-[0_0_0]! opacity-0 -translate-x-[18px] border-r-0 border-r-transparent transition-[flex-basis_1s_cubic-bezier(0.32,0.72,0,1),width_1s_cubic-bezier(0.32,0.72,0,1),opacity_0.6s_ease-out_0.05s,transform_1s_cubic-bezier(0.32,0.72,0,1),border-color_0.6s_ease]";

/// 已收起（宽度归零、不可交互）。
pub(crate) const SIDEBAR_HIDDEN: &str = "min-w-0! max-w-0! w-0! flex-[0_0_0]! overflow-hidden opacity-0 -translate-x-[18px] border-r-0 border-r-transparent pointer-events-none";

/// 侧栏滚动区（左上柔光 + 竖向渐变底）。
pub(crate) const SIDEBAR_SCROLL: &str = "flex-1 overflow-y-auto px-4 pt-0 pb-6 flex flex-col gap-6 bg-[radial-gradient(circle_at_10%_5%,var(--ac-color-4285f42e),transparent_20%),linear-gradient(180deg,var(--primary-color-1),var(--primary-color-1))]";

/// 顶部图标导航条（与中间标签栏同高）。
pub(crate) const SIDEBAR_NAV: &str = "flex flex-row items-center gap-1 shrink-0 box-border min-h-[28px]";

/// 导航图标按钮；激活态交给 `data-active`。
pub(crate) const SIDEBAR_NAV_ITEM: &str = "inline-flex items-center justify-center w-6 h-6 min-w-6 min-h-6 p-0 border-0 rounded-[5px] bg-transparent text-secondary-2 cursor-pointer transition-[color,background-color] duration-150 ease-out hover:bg-border-light data-[active=true]:text-brand";

/// 导航图标。
pub(crate) const SIDEBAR_NAV_ICON: &str = "block shrink-0";

/// 角色卡。`::after` 是悬停时的柔光层（`-z-10` + `isolate` 保证压在背景之上、内容之下）。
pub(crate) const AGENT_CARD: &str = "group relative isolate p-0 rounded-2xl cursor-default border border-border-light bg-primary shadow-[0_4px_14px_var(--shadow-color)] scale-[1.01] transition-[opacity,border-color,box-shadow,background] duration-200 ease-out backdrop-blur-[14px] backdrop-saturate-[1.12] after:[content:''] after:absolute after:inset-px after:rounded-2xl after:bg-[linear-gradient(135deg,var(--ac-color-4285f42e),var(--primary-color-a26))] after:opacity-0 after:transition-opacity after:duration-200 after:-z-10 hover:after:opacity-100 data-[active=true]:after:hidden dark:after:bg-transparent";

/// 非当前角色：淡渐变底（`:not(.is-active)` → `data-active=false`）。
pub(crate) const AGENT_CARD_IDLE: &str = "bg-[linear-gradient(158deg,color-mix(in_srgb,color-mix(in_srgb,var(--ac-color-eff6ff)_72%,var(--primary-color))_78%,transparent)_0%,color-mix(in_srgb,var(--primary-color-1)_78%,transparent)_48%,color-mix(in_srgb,var(--primary-color)_80%,transparent)_100%)] hover:bg-primary-3";

/// 当前角色：蓝向径向光 + 斜向渐变。
pub(crate) const AGENT_CARD_ACTIVE: &str = "opacity-100 border-border bg-[radial-gradient(95%_85%_at_0%_0%,color-mix(in_srgb,var(--ac-color-93c5fd)_24%,transparent)_0%,transparent_58%),linear-gradient(152deg,color-mix(in_srgb,var(--ac-color-eff6ff)_82%,transparent)_0%,color-mix(in_srgb,color-mix(in_srgb,var(--primary-color-3)_55%,var(--primary-color))_80%,transparent)_68%,color-mix(in_srgb,var(--primary-color)_84%,transparent)_88%)] shadow-[0_4px_18px_var(--shadow-color)] dark:bg-[var(--ac-agent-card-active-surface)] dark:shadow-[inset_0_1px_0_rgb(255_255_255/0.06)]";

/// 角色卡底部区域：选择钮与删除钮叠在同一格。
pub(crate) const AGENT_CARD_TOOLBAR: &str = "grid grid-cols-1 grid-rows-[auto]";

/// 铺满的选择区（右侧留出删除钮的位置）。
pub(crate) const AGENT_SELECT_HIT: &str = "col-start-1 row-start-1 min-w-0 self-stretch justify-self-stretch m-0 pl-2 pr-[calc(0.5rem+1.75rem+0.25rem)] py-[0.4rem] border-0 bg-transparent text-start font-[inherit] text-inherit cursor-pointer rounded-[0.65rem] box-border focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color-mix(in_srgb,var(--brand)_55%,transparent)]";

/// 删除钮（叠在选择区之上）。
pub(crate) const AGENT_DELETE: &str = "col-start-1 row-start-1 z-[1] justify-self-end self-center inline-flex items-center justify-center w-7 h-7 my-0 mr-[0.4rem] ml-0 p-0 border-0 rounded-lg bg-transparent text-primary-5 cursor-pointer transition-[color,background] duration-150 ease-out hover:text-[var(--primary-error-color-light)] hover:bg-[color-mix(in_srgb,var(--primary-error-color-light)_12%,transparent)] focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-[color-mix(in_srgb,var(--primary-error-color-light)_45%,transparent)] disabled:opacity-[0.38] disabled:cursor-not-allowed";

/// 删除图标。
pub(crate) const AGENT_DELETE_ICON: &str = "block";

/// 角色行（头像 + 文案）。
pub(crate) const AGENT_ROW: &str = "flex items-center gap-[0.625rem] min-h-9";

/// 头像圆角方块。
pub(crate) const AGENT_AVATAR: &str = "w-9 h-9 rounded-xl flex items-center justify-center overflow-hidden shrink-0";

/// 头像图片。
pub(crate) const AGENT_AVATAR_IMG: &str = "w-full h-full object-contain";

/// 角色名。
pub(crate) const AGENT_NAME: &str = "text-sm font-bold text-text-primary m-0 leading-[1.25]";

/// 文案容器（吃满剩余宽度）。
pub(crate) const AGENT_META: &str = "flex-1 min-w-0 flex items-center";

/// 「新建角色」按钮。
pub(crate) const BTN_NEW_AGENT: &str = "w-full flex items-center justify-center gap-[0.375rem] px-4 py-3 rounded-2xl border border-dashed border-primary-2 bg-primary-1 text-secondary-3 text-[0.75rem] font-bold cursor-default transition-[background,border-color,color] duration-150 ease-out hover:bg-[var(--ac-color-eff6ff)] hover:border-[var(--ac-color-93c5fd)] hover:text-brand";

/// 侧栏右侧拖拽热区。
pub(crate) const SIDEBAR_RESIZER: &str = "absolute top-0 bottom-0 -right-[5px] w-2 cursor-col-resize z-[5] bg-transparent transition-[background] duration-150 ease-out hover:bg-[linear-gradient(90deg,transparent,var(--ac-color-4285f42e)_40%,var(--ac-color-4285f42e))]";

/// 中间栏外壳。
pub(crate) const CENTER: &str = "flex-1 min-w-0 flex flex-col overflow-hidden bg-primary shadow-[-2px_4px_20px_var(--shadow-color)] opacity-100 translate-x-0 transition-[flex,opacity,transform] duration-[0.22s] ease-out group-data-[resizing=true]/root:[transition:none]!";

/// 中间栏收起中。
pub(crate) const CENTER_SHUTTING: &str = "flex-[0_0_0]! opacity-0 translate-x-[18px] pointer-events-none";

/// 中间栏已收起。
pub(crate) const CENTER_HIDDEN: &str = "flex-[0_0_0] opacity-0 translate-x-[18px] pointer-events-none";

/// 标签栏。
pub(crate) const CENTER_TABS: &str = "relative flex shrink-0 items-stretch gap-0 min-h-[28px] pt-0 pr-1 pb-0 pl-[6px] overflow-hidden border-b border-border-light bg-primary [&::-webkit-scrollbar]:h-[3px] dark:border-b-transparent dark:after:[content:''] dark:after:absolute dark:after:left-0 dark:after:right-0 dark:after:bottom-0 dark:after:h-px dark:after:z-[6] dark:after:pointer-events-none dark:after:opacity-[var(--ac-sand-opacity)] dark:after:bg-[var(--ac-sand-grad-x)]";

/// 标签横向滚动容器。
pub(crate) const CENTER_TABS_SCROLL: &str = "flex flex-1 min-w-0 items-stretch gap-0 overflow-x-auto overflow-y-hidden [scrollbar-width:thin]";

/// 单个标签。`data-active` / `data-preview` 驱动下划线、斜体与关闭钮显隐。
pub(crate) const CENTER_TAB: &str = "group/tab relative inline-flex items-center gap-[2px] max-w-[160px] min-w-0 px-2 py-[6px] border-0 rounded-none bg-transparent text-text-tertiary text-[0.75rem] font-medium leading-[1.2] cursor-pointer select-none transition-[color] duration-100 ease-out hover:text-text-secondary data-[active=true]:text-text-primary data-[active=true]:font-semibold data-[active=true]:after:[content:''] data-[active=true]:after:absolute data-[active=true]:after:left-2 data-[active=true]:after:right-2 data-[active=true]:after:-bottom-px data-[active=true]:after:h-[1.5px] data-[active=true]:after:bg-text-primary data-[active=true]:after:rounded-[1px] data-[active=true]:after:z-[1]";

/// 标签标题。
pub(crate) const CENTER_TAB_TITLE: &str = "min-w-0 overflow-hidden text-ellipsis whitespace-nowrap group-data-[preview=true]/tab:italic group-data-[preview=true]/tab:font-normal";

/// 标签右键菜单项里的文案。
pub(crate) const CENTER_TAB_CTX_LABEL: &str = "flex-auto min-w-0 text-left";

/// 标签右键菜单项里的快捷键。
pub(crate) const CENTER_TAB_CTX_KBD: &str = "shrink-0 ml-auto pl-4 text-[0.68rem] font-normal text-text-tertiary tracking-[0.02em] tabular-nums";

/// 未保存圆点。
pub(crate) const CENTER_TAB_DIRTY: &str = "shrink-0 text-[0.55rem] leading-none text-[var(--ac-color-f97316)] opacity-95";

/// 关闭钮（悬停标签或激活时浮现）。
pub(crate) const CENTER_TAB_CLOSE: &str = "inline-flex items-center justify-center shrink-0 w-[14px] h-[14px] m-0 p-0 border-0 rounded-[2px] bg-transparent text-text-tertiary opacity-0 cursor-pointer transition-[opacity,color,background] duration-[0.12s] ease-out group-hover/tab:opacity-55 group-data-[active=true]/tab:opacity-55 hover:opacity-100! hover:text-text-primary hover:bg-[color-mix(in_srgb,var(--secondary-color-3)_12%,transparent)]";

/// 关闭图标。
pub(crate) const CENTER_TAB_CLOSE_ICON: &str = "block";

/// 「Markdown 预览」切换钮。
pub(crate) const CENTER_TABS_MD_PREVIEW: &str = "inline-flex items-center justify-center shrink-0 self-center w-[1.55rem] h-[1.55rem] mx-[2px] my-0 ml-1 p-0 border-0 rounded-[0.35rem] bg-transparent text-text-tertiary cursor-pointer transition-[color,background] duration-[0.12s] ease-out hover:text-text-primary hover:bg-[color-mix(in_srgb,var(--secondary-color-3)_12%,transparent)]";

/// 中间栏空态整块（铺满，右键任意处可命中自定义菜单）。
pub(crate) const CENTER_EMPTY: &str = "flex-1 w-full min-h-full box-border flex flex-col items-center justify-center gap-4 px-4 py-8 text-center text-secondary-3";

/// 空态 logo 容器。
pub(crate) const CENTER_EMPTY_LOGO_WRAP: &str = "relative flex items-center justify-center";

/// 空态 logo。
pub(crate) const CENTER_EMPTY_LOGO: &str = "w-[5.5rem] h-[5.5rem] object-cover rounded-full shrink-0 cursor-default select-none";

/// 空态说明文案。
pub(crate) const CENTER_EMPTY_INTRO: &str = "m-0 max-w-[22rem] text-[0.95rem] font-medium leading-[1.55] tracking-[0.04em] text-secondary-3";

/// 空态操作区。
pub(crate) const CENTER_EMPTY_ACTIONS: &str = "flex flex-col flex-nowrap items-stretch justify-center gap-[0.45rem] w-[min(15.5rem,92vw)] mx-auto mt-[0.35rem] mb-0";

/// 空态操作行（左文案 + 右快捷键）。
pub(crate) const CENTER_EMPTY_OPEN_BTN: &str = "group/btn flex items-center justify-between gap-[0.85rem] w-full mt-0 pl-[0.85rem] pr-[0.7rem] py-2 min-h-[2.15rem] border-0 rounded-[0.45rem] bg-[#eef1f5] text-[#2a2f38] text-[0.8125rem] font-semibold tracking-[0.01em] leading-[1.2] cursor-default box-border shadow-none transition-[background,color] duration-150 ease-out hover:bg-[#d8dee8] hover:text-[#12151a] active:bg-[#c9d0db] dark:bg-[#1c1c20] dark:text-[#cccccc] dark:hover:bg-[#26262b] dark:hover:text-[#e8e8ec] dark:active:bg-[#2e2e34] motion-reduce:transition-none";

/// 空态操作行的文案。
pub(crate) const CENTER_EMPTY_OPEN_BTN_LABEL: &str = "flex-auto text-left whitespace-nowrap";

/// 空态操作行的快捷键容器。
pub(crate) const CENTER_EMPTY_OPEN_BTN_KEYS: &str = "inline-flex items-center gap-[0.22rem] shrink-0";

/// 单个按键帽（`kbd`）。悬停按钮时描边与字色加深。
pub(crate) const CENTER_EMPTY_KBD: &str = "inline-flex items-center justify-center min-w-[1.2rem] h-[1.2rem] px-[0.28rem] border border-[#c5cad3] border-b-[#a8b0bc] rounded-[0.28rem] bg-[linear-gradient(180deg,#ffffff_0%,#e9edf2_100%)] shadow-[inset_0_1px_0_rgb(255_255_255/0.95),0_1px_0_rgb(42_47_56/0.08)] text-[#5a6573] [font-family:inherit] text-[0.68rem] font-[650] leading-none group-hover/btn:text-[#3a4350] group-hover/btn:border-[#aeb5c0] dark:border-[#3a3a40] dark:border-b-[#2a2a2e] dark:bg-[linear-gradient(180deg,#2a2a2e_0%,#1a1a1d_100%)] dark:shadow-[inset_0_1px_0_rgb(255_255_255/0.06),0_1px_0_rgb(0_0_0/0.35)] dark:text-[#9d9d9d] dark:group-hover/btn:text-[#cccccc] dark:group-hover/btn:border-[#4a4a50]";

/// 内嵌浏览器面板。
pub(crate) const WEB_BROWSER_PANE: &str = "flex flex-col min-h-0 h-full w-full box-border overflow-hidden bg-[var(--primary-color-1,#f1f5f9)] dark:bg-[#141416]";

/// 内嵌浏览器地址栏。
pub(crate) const WEB_BROWSER_BAR: &str = "flex gap-2 items-center flex-none px-3 py-2 box-border";

/// 地址输入框。
pub(crate) const WEB_BROWSER_INPUT: &str = "flex-1 min-w-0 px-[0.85rem] py-[0.4rem] border border-border rounded-full bg-white text-inherit text-[0.8125rem] leading-[1.4] outline-none box-border transition-[border-color] duration-150 ease-out focus:border-border-hover dark:bg-[#1c1c20] dark:text-[#f5f5f7]";

/// 「前往」按钮（与聊天发送钮同色）。
pub(crate) const WEB_BROWSER_GO: &str = "flex-none px-[0.95rem] py-[0.4rem] border-0 rounded-full inline-flex items-center justify-center text-[0.8125rem] font-[650] text-white bg-[var(--search-button)] cursor-pointer transition-[background,color] duration-150 ease-out hover:bg-[var(--search-button-hover)]";

/// 页面 iframe。
pub(crate) const WEB_BROWSER_FRAME: &str = "flex-auto min-h-0 w-full border-0 bg-white";

/// 空页提示。
pub(crate) const WEB_BROWSER_EMPTY: &str = "flex-1 flex items-center justify-center p-4 text-[color-mix(in_srgb,currentColor_55%,transparent)] text-sm text-center [&_p]:m-0";

/// 终端不可用占位（web 无本地终端）。
pub(crate) const TERMINAL_UNAVAILABLE: &str = "flex-[1_1_auto] flex flex-col items-start justify-center gap-[6px] px-[14px] py-[16px] text-[12px] leading-[1.5] text-secondary-2 [font-family:'SF_Mono',Menlo,Monaco,ui-monospace,'Cascadia_Mono',Consolas,monospace]";

pub(crate) const TERMINAL_UNAVAILABLE_HINT: &str = "m-0 text-[12px] opacity-85";

// ——————————————————————————— P6：设置区弹窗 / 角色面板 / Markdown / 代码块 ———————————————————————————

/// 设置页主滚动容器（文件编辑器等全屏面板时切换为纵向布局）。
pub(crate) const MAIN_BODY: &str = "ac-main-body flex-1 min-h-0 w-full box-border m-0 p-[2rem] overflow-auto [scrollbar-width:thin] [scrollbar-color:var(--primary-color-2)_transparent] [&::-webkit-scrollbar]:w-2 [&::-webkit-scrollbar]:h-2 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:border-2 [&::-webkit-scrollbar-thumb]:border-solid [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:bg-[var(--primary-color-2)] [&::-webkit-scrollbar-thumb]:[background-clip:padding-box] [&::-webkit-scrollbar-thumb:hover]:bg-[var(--primary-color-5)] [&::-webkit-scrollbar-thumb:hover]:[background-clip:padding-box] has-[[data-file-editor]]:flex has-[[data-file-editor]]:flex-col has-[[data-file-editor]]:min-h-0 has-[[data-file-editor]]:p-0 has-[[data-file-editor]]:overflow-hidden has-[[data-preview-pane]]:flex has-[[data-preview-pane]]:flex-col has-[[data-preview-pane]]:min-h-0 has-[[data-preview-pane]]:p-0 has-[[data-preview-pane]]:overflow-hidden has-[[data-web-browser-pane]]:flex has-[[data-web-browser-pane]]:flex-col has-[[data-web-browser-pane]]:min-h-0 has-[[data-web-browser-pane]]:p-0 has-[[data-web-browser-pane]]:overflow-hidden has-[[data-file-diff]]:flex has-[[data-file-diff]]:flex-col has-[[data-file-diff]]:min-h-0 has-[[data-file-diff]]:p-0 has-[[data-file-diff]]:overflow-hidden has-[.ac-skills-detail-pane]:p-0";

/// API 配置弹窗遮罩（叠层弹窗加 DELETE_ROLE_MODAL_BACKDROP 附加段）。
pub(crate) const API_MODAL_BACKDROP: &str = "fixed inset-0 z-[1200] flex items-center justify-center p-[2rem] bg-[var(--shadow-color)] backdrop-blur-[10px]";

pub(crate) const DELETE_ROLE_MODAL_BACKDROP: &str = "z-[1230] [&_button]:w-full [&_button]:box-border [&_[data-desc]]:font-medium [&_[data-desc]]:tracking-[0.012em]";

pub(crate) const API_MODAL: &str = "w-[min(520px,90vw)] box-border px-[2.75rem] pt-[2.45rem] pb-[1.9rem] rounded-[1.8rem] bg-[linear-gradient(180deg,var(--primary-color),var(--primary-color-1))] border border-primary-2 shadow-[0_34px_90px_var(--shadow-color),inset_0_1px_0_var(--shadow-color)]";

pub(crate) const API_MODAL_TITLE: &str = "m-0 text-secondary-1 text-[1.55rem] leading-[1.2] font-black tracking-[-0.04em]";

/// 描述文案（`data-desc` 供叠层弹窗降级字重；strong 与正文同色）。
pub(crate) const API_MODAL_DESC: &str = "mt-[0.85rem] mb-[0.85rem] mx-0 text-secondary-3 text-[0.88rem] leading-[1.7] font-[650] [&_strong]:text-secondary-3";

pub(crate) const API_MODAL_FIELD: &str = "[&+&]:mt-[1.25rem]";

pub(crate) const AUTH_LABEL: &str = "flex flex-col gap-[0.35rem] mb-[0.85rem] text-[0.72rem] font-bold text-secondary-3";

pub(crate) const API_MODAL_INPUT: &str = "w-full h-[3rem] box-border rounded-[0.85rem] border border-border bg-primary-1 px-[1.15rem] py-0 text-secondary-1 [font:inherit] text-[0.92rem] font-[650] outline-none placeholder:font-medium placeholder:text-[var(--primary-color-4)] focus:border-[var(--ac-color-93c5fd)] focus:shadow-[0_0_0_4px_var(--shadow-color)]";

pub(crate) const API_MODAL_HELP: &str = "mt-[0.6rem] mb-0 ml-0 mr-0 text-[var(--primary-color-5)] text-[0.8rem] font-[650]";

pub(crate) const API_MODAL_ACTIONS: &str = "grid grid-cols-[1fr_1fr] gap-[1rem] mt-[2.25rem]";

pub(crate) const API_MODAL_CANCEL: &str = "h-[3rem] rounded-[0.9rem] border-0 inline-flex items-center justify-center text-[0.92rem] font-black text-secondary-1 bg-[var(--ac-color-eff6ff)] cursor-default";

pub(crate) const API_MODAL_SUBMIT: &str = "h-[3rem] rounded-[0.9rem] border-0 inline-flex items-center justify-center text-[0.92rem] font-black text-primary bg-[var(--button-primary)] shadow-[0_14px_32px_var(--shadow-color)] cursor-default";

// —— 角色面板（人格 / 插件 / 应用弹窗共用按钮）——

pub(crate) const ROLE_PANEL: &str = "w-full max-w-[1160px] mx-auto box-border px-[2.75rem] pt-[2.2rem] pb-[2rem] rounded-[1.15rem] bg-primary shadow-[0_26px_70px_var(--shadow-color),inset_0_1px_0_var(--shadow-color)]";

pub(crate) const ROLE_HEADER: &str = "flex items-start justify-between gap-[1.5rem]";

pub(crate) const ROLE_HEADING: &str = "flex items-center gap-[1.35rem] min-w-0 flex-1";

pub(crate) const ROLE_HEADING_TEXT: &str = "min-w-0 flex-1";

/// 头像（conic 光环由 before 伪元素 + halo 动画呈现）。
pub(crate) const ROLE_AVATAR: &str = "relative isolate w-[4rem] h-[4rem] rounded-[1.25rem] flex items-center justify-center overflow-hidden shrink-0 before:[content:''] before:absolute before:inset-[-8%] before:rounded-[inherit] before:opacity-[0.42] before:z-0 before:pointer-events-none before:animate-[ac-role-avatar-halo-spin_56s_linear_infinite] before:bg-[conic-gradient(from_0deg,color-mix(in_srgb,var(--border-color-light)_22%,transparent)_0deg,transparent_70deg,color-mix(in_srgb,var(--ac-color-ecfeff)_28%,transparent)_115deg,transparent_165deg,color-mix(in_srgb,var(--ac-color-93c5fd)_14%,transparent)_210deg,transparent_275deg,color-mix(in_srgb,var(--border-color-light)_18%,transparent)_320deg,transparent_360deg)]";

pub(crate) const ROLE_AVATAR_IMG: &str = "relative z-[1] w-full h-full object-contain";

pub(crate) const ROLE_NAME_INPUT: &str = "block w-full max-w-[28rem] m-0 ml-[-0.55rem] box-border border border-transparent rounded-[0.65rem] px-[0.55rem] py-[0.2rem] text-[1.55rem] leading-[1.2] font-extrabold tracking-[-0.03em] text-secondary-1 bg-transparent outline-none transition-[border-color,background-color,box-shadow] duration-150 hover:border-primary-2 hover:bg-primary-1 focus:border-border focus:bg-primary focus:shadow-[0_0_0_3px_var(--shadow-color)]";

pub(crate) const ROLE_SUBTITLE: &str = "mt-[0.35rem] mb-0 ml-0 mr-0 text-[0.92rem] font-bold text-secondary-3";

pub(crate) const ROLE_DIVIDER: &str = "h-px mt-[1.5rem] mb-[1.6rem] mx-0 bg-primary-2";

pub(crate) const ROLE_ACTIONS: &str = "flex justify-end gap-[1.15rem] mt-[1.75rem]";

/// 角色面板按钮基底（hover 浮起 / active 回弹 / disabled 归零）。
pub(crate) const ROLE_BTN: &str = "min-w-[11.4rem] h-[3.35rem] rounded-full border border-transparent inline-flex items-center justify-center gap-[0.7rem] text-[0.9rem] font-extrabold cursor-default transition-[transform,box-shadow,background-color,border-color,filter] duration-150 hover:-translate-y-[2px] active:translate-y-px disabled:opacity-55 disabled:cursor-not-allowed disabled:transform-none disabled:hover:transform-none disabled:active:transform-none";

pub(crate) const ROLE_BTN_ICON: &str = "inline-block shrink-0";

pub(crate) const ROLE_BTN_SECONDARY: &str = "text-secondary-1 border-primary-2 bg-[linear-gradient(180deg,var(--primary-color)_0%,var(--primary-color-3)_100%)] shadow-[0_10px_24px_var(--shadow-color)] hover:shadow-[0_18px_38px_var(--shadow-color)]";

/// 主按钮（`data-save` 时人格保存改用发送钮可发送态底色 token）。
pub(crate) const ROLE_BTN_PRIMARY: &str = "text-white border-transparent bg-[var(--button-primary)] shadow-[0_10px_28px_color-mix(in_srgb,var(--button-primary)_36%,transparent)] hover:bg-[var(--button-primary-light)] hover:shadow-[0_16px_36px_color-mix(in_srgb,var(--button-primary)_44%,transparent)] data-[save=true]:text-primary data-[save=true]:bg-[var(--ac-chat-send-ready)] data-[save=true]:shadow-[0_10px_28px_color-mix(in_srgb,var(--ac-chat-send-ready)_36%,transparent)] data-[save=true]:hover:bg-[var(--ac-chat-send-ready-hover)] data-[save=true]:hover:shadow-[0_16px_36px_color-mix(in_srgb,var(--ac-chat-send-ready)_44%,transparent)] data-[save=true]:focus-visible:bg-[var(--ac-chat-send-ready-hover)] data-[save=true]:focus-visible:shadow-[0_16px_36px_color-mix(in_srgb,var(--ac-chat-send-ready)_44%,transparent)]";

pub(crate) const PERSONA_PROMPT_CARD: &str = "mt-[1.75rem] p-0 border-0 rounded-none bg-transparent shadow-none";

pub(crate) const PERSONA_PROMPT_HEADER: &str = "flex items-center justify-between gap-[1rem] mb-[1rem]";

pub(crate) const PERSONA_PROMPT_TITLE: &str = "inline-flex items-center gap-[0.65rem] text-secondary-1 text-[1rem] font-extrabold";

pub(crate) const PERSONA_PROMPT_ICON: &str = "shrink-0";

pub(crate) const PERSONA_TEXTAREA: &str = "w-full min-h-[10.6rem] resize-y box-border px-[1.5rem] py-[1.35rem] rounded-[0.95rem] border border-primary-2 bg-primary-1 text-secondary-3 [font:inherit] text-[1rem] leading-[1.75] font-normal outline-none focus:border-border focus:shadow-[0_0_0_4px_var(--shadow-color)]";

// —— 已安装技能按钮（学习 / 忘却 / 移除）——

/// 基底：与技能市场一键下载同等规格的实心按钮。
pub(crate) const INSTALLED_BTN: &str = "h-[2.4rem] min-w-[6rem] px-[0.9rem] py-0 rounded-[0.8rem] border border-transparent inline-flex items-center justify-center gap-[0.4rem] text-[0.8rem] font-extrabold tracking-[0.01em] cursor-pointer box-border transition-[background-color,color,box-shadow,transform] duration-150 [&_svg]:opacity-90 disabled:[&_svg]:opacity-85";

/// 学习（未装备）：浅色描边渐变次级按钮。
pub(crate) const INSTALLED_BTN_LEARN: &str = "text-secondary-1 border-primary-2 bg-[linear-gradient(180deg,var(--primary-color)_0%,var(--primary-color-3)_100%)] shadow-[0_10px_24px_var(--shadow-color)] hover:not-disabled:shadow-[0_16px_32px_var(--shadow-color)] hover:not-disabled:-translate-y-px active:not-disabled:translate-y-0 active:not-disabled:shadow-[0_8px_18px_var(--shadow-color)] focus-visible:outline-2 focus-visible:outline-[color-mix(in_srgb,var(--brand)_40%,transparent)] focus-visible:outline-offset-2 disabled:opacity-55 disabled:cursor-wait disabled:shadow-none disabled:transform-none";

/// 忘却（已装备，`data-on`）：高对比主色按钮。
pub(crate) const INSTALLED_BTN_ON: &str = "data-[on=true]:text-primary data-[on=true]:bg-[var(--button-primary)] data-[on=true]:border-transparent data-[on=true]:shadow-[0_12px_28px_var(--shadow-color)] data-[on=true]:hover:not-disabled:bg-[var(--button-primary-light)] data-[on=true]:hover:not-disabled:-translate-y-px data-[on=true]:hover:not-disabled:shadow-[0_18px_38px_var(--shadow-color)] data-[on=true]:active:not-disabled:bg-[color-mix(in_srgb,var(--button-primary-light)_85%,black)] data-[on=true]:active:not-disabled:translate-y-0 data-[on=true]:active:not-disabled:shadow-[0_8px_18px_var(--shadow-color)]";

/// 移除：浅红次级按钮（未忘却时禁用仅降透明度）。
pub(crate) const INSTALLED_BTN_REMOVE: &str = "text-[var(--ac-color-991b1b)] bg-[color-mix(in_srgb,var(--ac-color-991b1b)_8%,var(--primary-color))] shadow-none hover:not-disabled:bg-[color-mix(in_srgb,var(--ac-color-991b1b)_14%,var(--primary-color))] hover:not-disabled:-translate-y-px active:not-disabled:bg-[color-mix(in_srgb,var(--ac-color-991b1b)_22%,var(--primary-color))] active:not-disabled:translate-y-0 focus-visible:outline-2 focus-visible:outline-[color-mix(in_srgb,var(--ac-color-991b1b)_38%,var(--brand))] focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:shadow-none disabled:transform-none disabled:opacity-55";

/// 技能市场按钮（可下载 / 已安装）。
pub(crate) const SKILL_INSTALL_BTN: &str = "h-[2.4rem] min-w-[6rem] px-[0.9rem] py-0 rounded-[0.8rem] border border-transparent inline-flex items-center justify-center gap-[0.4rem] text-[0.8rem] font-extrabold text-primary bg-[var(--button-primary)] shadow-[0_12px_28px_var(--shadow-color)] cursor-default";

pub(crate) const SKILL_INSTALLED_BTN: &str = "h-[2.4rem] min-w-[6rem] px-[0.9rem] py-0 rounded-[0.8rem] border border-transparent inline-flex items-center justify-center gap-[0.4rem] text-[0.8rem] font-extrabold text-secondary-3 bg-[var(--ac-color-eff6ff)] cursor-default";

pub(crate) const SKILL_SOURCE: &str = "inline-flex items-center w-fit mb-[0.45rem] px-[0.5rem] py-[0.18rem] rounded-full text-[var(--ac-color-2563eb)] bg-[var(--ac-color-eff6ff)] text-[0.66rem] font-black";

pub(crate) const SKILL_MARKET_META: &str = "flex flex-wrap gap-[0.45rem] mt-[1rem] [&>span]:px-[0.5rem] [&>span]:py-[0.22rem] [&>span]:rounded-full [&>span]:text-secondary-3 [&>span]:bg-primary-1 [&>span]:text-[0.68rem] [&>span]:font-[850]";

pub(crate) const SKILL_CARD_ICON: &str = "shrink-0";

pub(crate) const INSTALLED_EMPTY_ICON: &str = "opacity-45 text-[var(--primary-color-5)]";

// —— 聊天 Markdown（后代元素用任意选择器挂在容器上）——

pub(crate) const CHAT_MD: &str = "ac-chat-md font-medium [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_p]:m-[0.45em_0] [&_h1]:m-[0.85em_0_0.4em] [&_h2]:m-[0.85em_0_0.4em] [&_h3]:m-[0.85em_0_0.4em] [&_h4]:m-[0.85em_0_0.4em] [&_h1]:font-[750] [&_h2]:font-[750] [&_h3]:font-[750] [&_h4]:font-[750] [&_h1]:leading-[1.35] [&_h2]:leading-[1.35] [&_h3]:leading-[1.35] [&_h4]:leading-[1.35] [&_h1]:text-[1.05rem] [&_h2]:text-[0.98rem] [&_h3]:text-[0.92rem] [&_h4]:text-[0.92rem] [&_h1]:text-text-primary [&_h2]:text-text-primary [&_h3]:text-text-primary [&_h4]:text-text-primary [&_strong]:font-[750] [&_strong]:text-text-primary [&_em]:italic [&_hr]:m-[0.75em_0] [&_hr]:border-0 [&_hr]:border-t [&_hr]:border-t-solid [&_hr]:border-t-[var(--border-color-light)] [&_ul]:m-[0.4em_0] [&_ul]:pl-[1.25em] [&_ol]:m-[0.4em_0] [&_ol]:pl-[1.25em] [&_li]:m-[0.2em_0] [&_li>p]:m-[0.15em_0] [&_code]:px-[0.35em] [&_code]:py-[0.1em] [&_code]:rounded-[0.3rem] [&_code]:text-[0.9em] [&_code]:bg-[color-mix(in_srgb,var(--button-secondary)_65%,transparent)] [&_code]:[font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace] [&_pre]:m-[0.55em_0] [&_pre]:px-[0.75rem] [&_pre]:py-[0.65rem] [&_pre]:overflow-x-auto [&_pre]:rounded-[0.45rem] [&_pre]:text-[0.78rem] [&_pre]:leading-[1.5] [&_pre]:bg-[color-mix(in_srgb,var(--button-secondary)_75%,transparent)] [&_pre_code]:p-0 [&_pre_code]:bg-transparent [&_blockquote]:m-[0.5em_0] [&_blockquote]:pl-[0.75rem] [&_blockquote]:border-l-[3px] [&_blockquote]:border-l-solid [&_blockquote]:border-l-[var(--border-color)] [&_blockquote]:text-[var(--text-secondary)] [&_a]:text-brand [&_a]:underline [&_a]:underline-offset-[2px] [&_img]:inline-block [&_img]:w-[3.25rem] [&_img]:h-[3.25rem] [&_img]:object-cover [&_img]:align-middle [&_img]:mt-0 [&_img]:mr-[0.22rem] [&_img]:mb-[0.12rem] [&_img]:ml-0 [&_img]:rounded-[0.45rem] [&_img]:border [&_img]:border-primary-2 [&_img]:bg-primary-1 [&_table]:w-max [&_table]:min-w-full [&_table]:m-0 [&_table]:border-collapse [&_table]:text-[0.8rem] [&_th]:px-[0.5rem] [&_th]:py-[0.35rem] [&_td]:px-[0.5rem] [&_td]:py-[0.35rem] [&_th]:border [&_td]:border [&_th]:border-[var(--border-color-light)] [&_td]:border-[var(--border-color-light)] [&_th]:text-left [&_td]:text-left [&_th]:font-bold [&_th]:bg-[color-mix(in_srgb,var(--button-secondary)_50%,transparent)]";

/// 错误消息形态（图标前置 + 正文占满）。
pub(crate) const CHAT_MD_ERROR: &str = "flex items-start gap-[0.4rem] [&_.ac-chat-md-body]:flex-1 [&_.ac-chat-md-body]:min-w-0";

pub(crate) const CHAT_ERROR_ICON: &str = "shrink-0 inline-flex items-center justify-center mt-[0.12em] text-[var(--ac-chat-error-icon)] leading-none";

pub(crate) const CHAT_MD_BODY: &str = "ac-chat-md-body flex-1 min-w-0";

pub(crate) const CHAT_MD_CHUNK: &str = "[&>*:first-child]:mt-0 [&>*:last-child]:mb-0";

// —— 代码块（带工具条）——

pub(crate) const CHAT_CODE: &str = "m-[0.55em_0] rounded-[0.375rem] overflow-hidden border border-primary-2 bg-primary";

pub(crate) const CHAT_CODE_TOOLBAR: &str = "flex items-center justify-between gap-[0.5rem] min-h-[1.85rem] pt-[0.2rem] pr-[0.35rem] pb-[0.2rem] pl-[0.7rem] border-b border-[color-mix(in_srgb,var(--primary-color-2)_88%,transparent)] bg-primary-1";

pub(crate) const CHAT_CODE_META: &str = "min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-[0.65rem] font-medium tracking-[0.01em] leading-[1.2] text-[var(--text-tertiary)] [font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace]";

pub(crate) const CHAT_CODE_ACTIONS: &str = "flex shrink-0 items-center gap-[0.1rem] relative z-[2] pointer-events-auto";

pub(crate) const CHAT_CODE_BTN: &str = "inline-flex items-center gap-[0.28rem] px-[0.4rem] py-[0.22rem] border-0 rounded-[0.25rem] text-[0.66rem] font-medium leading-none text-[var(--text-tertiary)] bg-transparent cursor-pointer pointer-events-auto transition-[color,background-color] duration-150 [&_svg]:shrink-0 [&_svg]:opacity-85 hover:text-text-primary hover:bg-[color-mix(in_srgb,var(--primary-color-2)_55%,transparent)] hover:[&_svg]:opacity-100 focus-visible:outline-2 focus-visible:outline-[color-mix(in_srgb,var(--brand)_35%,transparent)] focus-visible:outline-offset-[1px]";

pub(crate) const CHAT_CODE_BTN_LABEL: &str = "tracking-[0.01em]";

pub(crate) const CHAT_CODE_PRE: &str = "m-0 px-[0.85rem] py-[0.7rem] overflow-x-auto text-[0.78rem] leading-[1.55] text-secondary-1 bg-primary [tab-size:2] [&_code]:p-0 [&_code]:bg-transparent [&_code]:text-inherit [&_code]:leading-[inherit] [&_code]:[font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace]";
