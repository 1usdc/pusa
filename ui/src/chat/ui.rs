//! 聊天区本地 UI 状态（与 [`protocol`] 传输类型分离）。

use protocol::AgentRunDetailDto;

/// 思考面板整体状态。
#[derive(Clone, PartialEq)]
pub enum ThinkingStatus {
    Running,
    Done,
    Error,
}

/// 单步阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPhase {
    Thinking,
    Observing,
    Done,
    Error,
}

/// 工具调用行状态。
#[derive(Clone, PartialEq)]
pub enum ToolStatus {
    Running,
    Done,
}

#[derive(Clone, PartialEq)]
pub struct UiThinkingTool {
    pub id: String,
    pub name: String,
    pub args: String,
    pub summary: Option<String>,
    pub status: ToolStatus,
}

#[derive(Clone, PartialEq)]
pub struct UiThinkingStep {
    pub index: usize,
    pub phase: StepPhase,
    pub model_text: String,
    pub tools: Vec<UiThinkingTool>,
    pub duration_ms: Option<i64>,
}

#[derive(Clone, PartialEq)]
pub struct UiAgentThinking {
    pub status: ThinkingStatus,
    /// 步骤详情是否展开（运行中强制展开；结束后可折叠）。
    pub expanded: bool,
    pub steps: Vec<UiThinkingStep>,
    /// 整轮 Agent 耗时（历史回填或从 step 汇总）。
    pub total_duration_ms: Option<i64>,
    /// 历史消息是否已请求过 agent run 详情回填。
    pub hydrate_requested: bool,
    /// 已废弃：回答到达后仍保留时间线；保留字段以免大面积改动序列化兼容。
    pub dismissing: bool,
    pub dismissed: bool,
}

impl UiAgentThinking {
    pub fn running() -> Self {
        Self {
            status: ThinkingStatus::Running,
            expanded: true,
            steps: Vec::new(),
            total_duration_ms: None,
            hydrate_requested: false,
            dismissing: false,
            dismissed: false,
        }
    }

    pub fn idle() -> Self {
        Self {
            status: ThinkingStatus::Done,
            expanded: false,
            steps: Vec::new(),
            total_duration_ms: None,
            hydrate_requested: false,
            dismissing: false,
            dismissed: false,
        }
    }

    pub fn from_agent_run_detail(detail: &AgentRunDetailDto) -> Self {
        let steps: Vec<UiThinkingStep> = detail
            .steps
            .iter()
            .map(|step| {
                let phase = match step.phase.as_str() {
                    "thinking" => StepPhase::Thinking,
                    "done" => StepPhase::Done,
                    "error" => StepPhase::Error,
                    _ => StepPhase::Observing,
                };
                UiThinkingStep {
                    index: step.index,
                    phase,
                    model_text: step.model_output.clone(),
                    tools: step
                        .tools
                        .iter()
                        .map(|tool| UiThinkingTool {
                            id: tool.tool_call_id.clone(),
                            name: tool.name.clone(),
                            args: tool.args.clone(),
                            summary: if tool.summary.trim().is_empty() {
                                None
                            } else {
                                Some(tool.summary.clone())
                            },
                            status: ToolStatus::Done,
                        })
                        .collect(),
                    duration_ms: step.duration_ms,
                }
            })
            .collect();
        let summed: i64 = steps.iter().filter_map(|s| s.duration_ms).sum();
        let total_duration_ms = detail
            .total_duration_ms
            .or(if summed > 0 { Some(summed) } else { None });
        Self {
            status: if detail.status == "completed" {
                ThinkingStatus::Done
            } else if detail.status == "error" {
                ThinkingStatus::Error
            } else {
                ThinkingStatus::Running
            },
            // 历史默认折叠，点摘要展开。
            expanded: false,
            steps,
            total_duration_ms,
            hydrate_requested: true,
            dismissing: false,
            dismissed: false,
        }
    }

    pub fn has_visible_content(&self) -> bool {
        !self.steps.is_empty()
            || matches!(self.status, ThinkingStatus::Running)
            || self.total_duration_ms.is_some()
    }

    /// 优先用落库总耗时，否则汇总各 step。
    pub fn resolved_duration_ms(&self) -> Option<i64> {
        if let Some(ms) = self.total_duration_ms {
            return Some(ms);
        }
        let sum: i64 = self.steps.iter().filter_map(|s| s.duration_ms).sum();
        if sum > 0 {
            Some(sum)
        } else {
            None
        }
    }
}

/// 单条气泡展示。
#[derive(Clone, PartialEq)]
pub enum UiChatMessage {
    User {
        id: Option<i64>,
        content: String,
        /// 本轮发送时的附件预览（历史加载为空）。
        attachments: Vec<crate::chat::ChatPendingAttachment>,
    },
    Assistant {
        id: Option<i64>,
        content: String,
        agent_run_id: Option<i64>,
        thinking: UiAgentThinking,
    },
}
