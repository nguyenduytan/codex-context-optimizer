use anyhow::{bail, Context, Result};
use ctxc_protocol::{ExecutionPolicy, Mode, ReasoningLevel};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub compiler: Compiler,
    pub reasoning: Reasoning,
    pub context: ContextConfig,
    pub agents: Agents,
    pub execution: Execution,
    pub output: Output,
    pub privacy: Privacy,
    pub trace: Trace,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Compiler {
    pub mode: String,
    pub preserve_negations: bool,
    pub preserve_paths: bool,
    pub preserve_literals: bool,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Reasoning {
    pub default: ReasoningLevel,
    pub complex: ReasoningLevel,
    pub max_auto: ReasoningLevel,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    pub project_map_max_bytes: usize,
    pub tool_output_token_limit: usize,
    pub budget: usize,
    pub reserve: usize,
    pub history_budget: usize,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Agents {
    pub enabled: bool,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Execution {
    pub prefer_targeted_search: bool,
    pub prefer_narrow_tests: bool,
    pub avoid_unrequested_web: bool,
    pub stop_when_acceptance_met: bool,
    pub max_model_retries: usize,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Output {
    pub verbosity: String,
    pub reasoning_summary: String,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Privacy {
    pub telemetry: bool,
    pub store_prompts: bool,
    pub store_responses: bool,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Trace {
    pub enabled: bool,
    pub retain_runs: usize,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            compiler: Compiler::default(),
            reasoning: Reasoning::default(),
            context: ContextConfig::default(),
            agents: Agents::default(),
            execution: Execution::default(),
            output: Output::default(),
            privacy: Privacy::default(),
            trace: Trace::default(),
            unknown: BTreeMap::new(),
        }
    }
}
impl Default for Compiler {
    fn default() -> Self {
        Self {
            mode: "auto".into(),
            preserve_negations: true,
            preserve_paths: true,
            preserve_literals: true,
            unknown: BTreeMap::new(),
        }
    }
}
impl Default for Reasoning {
    fn default() -> Self {
        Self {
            default: ReasoningLevel::Low,
            complex: ReasoningLevel::Medium,
            max_auto: ReasoningLevel::Medium,
            unknown: BTreeMap::new(),
        }
    }
}
impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            project_map_max_bytes: 8192,
            tool_output_token_limit: 4000,
            budget: 12000,
            reserve: 2000,
            history_budget: 0,
            unknown: BTreeMap::new(),
        }
    }
}
impl Default for Execution {
    fn default() -> Self {
        Self {
            prefer_targeted_search: true,
            prefer_narrow_tests: true,
            avoid_unrequested_web: true,
            stop_when_acceptance_met: true,
            max_model_retries: 0,
            unknown: BTreeMap::new(),
        }
    }
}
impl Default for Output {
    fn default() -> Self {
        Self {
            verbosity: "low".into(),
            reasoning_summary: "none".into(),
            unknown: BTreeMap::new(),
        }
    }
}
impl Default for Trace {
    fn default() -> Self {
        Self {
            enabled: true,
            retain_runs: 50,
            unknown: BTreeMap::new(),
        }
    }
}

pub fn parse_mode(s: &str) -> Result<Option<Mode>> {
    Ok(match s {
        "auto" => None,
        "chat" => Some(Mode::Chat),
        "micro" => Some(Mode::Micro),
        "code" => Some(Mode::Code),
        "complex" => Some(Mode::Complex),
        _ => bail!("mode must be auto|chat|micro|code|complex"),
    })
}
pub fn parse_effort(s: &str) -> Result<Option<ReasoningLevel>> {
    Ok(match s {
        "auto" => None,
        "minimal" => Some(ReasoningLevel::Minimal),
        "low" => Some(ReasoningLevel::Low),
        "medium" => Some(ReasoningLevel::Medium),
        "high" => Some(ReasoningLevel::High),
        "extreme" => Some(ReasoningLevel::Extreme),
        _ => bail!("effort must be auto|minimal|low|medium|high|extreme"),
    })
}
impl Config {
    pub fn policy(&self) -> Result<ExecutionPolicy> {
        if self.version != 1 {
            bail!("unsupported config schema {}; supported: 1. Back up the file and migrate explicitly; it was not modified.", self.version);
        }
        if !self.compiler.preserve_negations
            || !self.compiler.preserve_paths
            || !self.compiler.preserve_literals
        {
            bail!("v0.1 does not allow disabling fidelity protections");
        }
        if self.privacy.telemetry || self.privacy.store_prompts || self.privacy.store_responses {
            bail!("v0.1 supports only metadata storage and no telemetry; privacy settings must be false");
        }
        if self.execution.max_model_retries != 0 {
            bail!("v0.1 supports max_model_retries = 0 only");
        }
        if !self.execution.prefer_targeted_search
            || !self.execution.prefer_narrow_tests
            || !self.execution.stop_when_acceptance_met
        {
            bail!("v0.1 requires targeted search/tests and stop-at-acceptance policies");
        }
        if self.output.reasoning_summary != "none" {
            bail!("v0.1 requires output.reasoning_summary = 'none'");
        }
        if self.context.history_budget != 0 {
            bail!("persistent history is not implemented in v0.1; history_budget must be 0");
        }
        if self.trace.retain_runs > 1000 {
            bail!("trace.retain_runs must be <= 1000");
        }
        if self.context.project_map_max_bytes > 1024 * 1024 {
            bail!("project map maximum is 1 MiB");
        }
        let policy = ExecutionPolicy {
            mode: parse_mode(&self.compiler.mode)?,
            default_reasoning: self.reasoning.default,
            complex_reasoning: self.reasoning.complex,
            max_auto_reasoning: self.reasoning.max_auto,
            agents: self.agents.enabled,
            verbosity: self.output.verbosity.clone(),
            context_budget: self.context.budget,
            reserve_tokens: self.context.reserve,
            project_map_max_bytes: self.context.project_map_max_bytes,
            tool_output_token_limit: self.context.tool_output_token_limit,
            ..ExecutionPolicy::default()
        };
        ctxc_core::validate_policy(&policy)?;
        Ok(policy)
    }
    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = vec![];
        for (section, keys) in [
            ("", &self.unknown),
            ("compiler.", &self.compiler.unknown),
            ("reasoning.", &self.reasoning.unknown),
            ("context.", &self.context.unknown),
            ("agents.", &self.agents.unknown),
            ("execution.", &self.execution.unknown),
            ("output.", &self.output.unknown),
            ("privacy.", &self.privacy.unknown),
            ("trace.", &self.trace.unknown),
        ] {
            warnings.extend(
                keys.keys()
                    .map(|key| format!("Unknown config key {section}{key}; ignored.")),
            );
        }
        if !self.execution.avoid_unrequested_web {
            warnings.push("execution.avoid_unrequested_web=false does not enable web search; use --web on explicitly.".into());
        }
        warnings
    }
}

pub fn user_config_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("CTXC_USER_CONFIG") {
        return Some(PathBuf::from(p));
    }
    if let Some(p) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(p).join("ctxc/config.toml"));
    }
    let key = if cfg!(windows) { "APPDATA" } else { "HOME" };
    std::env::var_os(key).map(|p| {
        if cfg!(windows) {
            PathBuf::from(p).join("ctxc/config.toml")
        } else {
            PathBuf::from(p).join(".config/ctxc/config.toml")
        }
    })
}

pub fn merge(base: &mut toml::Value, incoming: toml::Value) {
    if let (Some(dst), Some(src)) = (base.as_table_mut(), incoming.as_table()) {
        for (key, value) in src {
            if let Some(old) = dst.get_mut(key) {
                merge(old, value.clone());
            } else {
                dst.insert(key.clone(), value.clone());
            }
        }
    } else {
        *base = incoming;
    }
}

pub fn load(root: &Path) -> Result<(Config, Vec<PathBuf>)> {
    let mut value = toml::Value::try_from(Config::default())?;
    let mut sources = vec![];
    for path in user_config_path()
        .into_iter()
        .chain(std::iter::once(root.join(".ctxc/config.toml")))
    {
        if path.is_file() {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("read config {}", path.display()))?;
            merge(
                &mut value,
                toml::from_str(&text)
                    .with_context(|| format!("parse config {}", path.display()))?,
            );
            sources.push(path);
        }
    }
    let config: Config = value.try_into()?;
    config.policy()?;
    Ok((config, sources))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_roundtrip() {
        let s = toml::to_string(&Config::default()).unwrap();
        let c: Config = toml::from_str(&s).unwrap();
        assert_eq!(
            c.policy().unwrap().max_auto_reasoning,
            ReasoningLevel::Medium
        );
    }
    #[test]
    fn unknown_warns() {
        let c: Config = toml::from_str("version=1\n[reasoning]\ntypo=true").unwrap();
        assert_eq!(
            c.warnings(),
            vec!["Unknown config key reasoning.typo; ignored."]
        );
    }
    #[test]
    fn merge_partial() {
        let mut base = toml::Value::try_from(Config::default()).unwrap();
        merge(&mut base, toml::from_str("[context]\nbudget=6000").unwrap());
        let c: Config = base.try_into().unwrap();
        assert_eq!(c.context.budget, 6000);
        assert_eq!(c.context.reserve, 2000);
    }
    #[test]
    fn unsafe_config_rejected() {
        for text in [
            "version=99",
            "[privacy]\ntelemetry=true",
            "[execution]\nmax_model_retries=3",
            "[reasoning]\nmax_auto='high'",
        ] {
            let c: Config = toml::from_str(text).unwrap();
            assert!(c.policy().is_err());
        }
    }
}
