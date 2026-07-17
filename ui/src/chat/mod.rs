//! 对话：流式传输、门面与本地气泡模型。
mod attachments;
mod error_text;
mod facade;
mod markdown;
mod thinking;
#[cfg(target_arch = "wasm32")]
pub(crate) mod transport_wasm;
mod ui;

pub use attachments::{attachment_from_file_data, ChatPendingAttachment};
#[cfg(target_arch = "wasm32")]
pub use attachments::attachment_from_bytes;
#[cfg(not(target_arch = "wasm32"))]
pub use attachments::desktop_paste;
#[cfg(target_arch = "wasm32")]
pub use attachments::wasm_paste;
pub use error_text::friendly_chat_error_message;
#[cfg(target_arch = "wasm32")]
pub use facade::api_base_url;
pub use facade::{
    activate_role, create_conversation, create_role, delete_conversation, delete_role,
    install_skill, list_chat_models, list_conversations, load_agent_run_detail,
    load_conversation_messages, load_equipped_skills, load_installed_skills, load_roles,
    load_skill_market, run_chat_turn, toggle_skill_equip, uninstall_installed_skill,
    update_conversation_title, update_role,
};
pub use markdown::ChatMarkdownBody;
pub use thinking::{ChatThinkingHydrator, ChatThinkingPanel, TraceFileOpen};
pub use ui::{
    StepPhase, ThinkingStatus, ToolStatus, UiAgentThinking, UiChatMessage, UiThinkingStep,
    UiThinkingTool,
};
