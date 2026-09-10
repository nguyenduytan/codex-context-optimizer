//! Provider-neutral, versioned request/result protocol. No I/O or global state.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PROTOCOL_VERSION: u32 = 1;

macro_rules! enum_type {
    ($name:ident { $($variant:ident),+ }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }
    };
}
enum_type!(Mode {
    Chat,
    Micro,
    Code,
    Complex
});
enum_type!(ReasoningLevel {
    Minimal,
    Low,
    Medium,
    High,
    Extreme
});
enum_type!(Preserve {
    Exact,
    Semantic,
    Compressible,
    Droppable
});
enum_type!(Lifecycle {
    Turn,
    Session,
    Task,
    Project,
    Persistent
});
enum_type!(Status {
    Active,
    Resolved,
    Superseded,
    Stale
});
enum_type!(BlockKind {
    Objective,
    Constraint,
    Acceptance,
    Code,
    FileReference,
    Error,
    ToolResult,
    Decision,
    Fact,
    UserPreference,
    OpenQuestion,
    CompletedWork,
    NextAction,
    Conversation,
    ProjectMap,
    ExternalReference
});

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Block {
    pub id: String,
    pub kind: BlockKind,
    pub source: String,
    pub content: String,
    pub preserve: Preserve,
    pub priority: f64,
    pub relevance: f64,
    pub freshness: f64,
    pub dependency: f64,
    pub user_weight: f64,
    pub token_estimate: usize,
    pub lifecycle: Lifecycle,
    pub status: Status,
    pub metadata: BTreeMap<String, serde_json::Value>,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: BlockKind::Fact,
            source: "user".into(),
            content: String::new(),
            preserve: Preserve::Compressible,
            priority: 0.5,
            relevance: 0.5,
            freshness: 1.0,
            dependency: 0.5,
            user_weight: 0.5,
            token_estimate: 0,
            lifecycle: Lifecycle::Task,
            status: Status::Active,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExecutionPolicy {
    pub mode: Option<Mode>,
    pub effort: Option<ReasoningLevel>,
    pub default_reasoning: ReasoningLevel,
    pub complex_reasoning: ReasoningLevel,
    pub max_auto_reasoning: ReasoningLevel,
    pub agents: bool,
    pub verbosity: String,
    pub web: String,
    pub context_budget: usize,
    pub reserve_tokens: usize,
    pub project_map_max_bytes: usize,
    pub tool_output_token_limit: usize,
}
impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            mode: None,
            effort: None,
            default_reasoning: ReasoningLevel::Low,
            complex_reasoning: ReasoningLevel::Medium,
            max_auto_reasoning: ReasoningLevel::Medium,
            agents: false,
            verbosity: "low".into(),
            web: "auto".into(),
            context_budget: 12000,
            reserve_tokens: 2000,
            project_map_max_bytes: 8192,
            tool_output_token_limit: 4000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompileInput {
    pub version: u32,
    pub request: String,
    #[serde(default)]
    pub context: Vec<Block>,
    #[serde(default)]
    pub policy: ExecutionPolicy,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub action: String,
    pub block_id: Option<String>,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimate {
    pub tokens: usize,
    pub exact: bool,
    pub method: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeEstimate {
    pub input_before: usize,
    pub input_after: usize,
    pub exact: bool,
    pub method: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledContext {
    pub version: u32,
    pub classification: Mode,
    pub reasoning: ReasoningLevel,
    pub contract: String,
    pub blocks: Vec<Block>,
    pub policy: ExecutionPolicy,
    pub estimate: SizeEstimate,
    pub trace: Vec<TraceEvent>,
    pub budget_exceeded: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskLedger {
    pub goal: Vec<String>,
    pub constraints: Vec<String>,
    pub active_files: Vec<String>,
    pub findings: Vec<String>,
    pub completed: Vec<String>,
    pub open: Vec<String>,
    pub next: Vec<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub input_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
}
