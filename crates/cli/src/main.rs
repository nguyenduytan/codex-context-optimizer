mod config;
mod project;
mod storage;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use ctxc_codex_adapter as adapter;
use ctxc_protocol::*;
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Parser, Debug)]
#[command(
    name = "ctxc",
    version,
    about = "Codex Context Optimizer — less context, more signal",
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Shorthand for ctxc run "task". Use '-' to read stdin.
    prompt: Option<String>,
    #[arg(long, global=true, value_parser=["auto","chat","micro","code","complex"])]
    mode: Option<String>,
    #[arg(long, global=true, value_parser=["auto","minimal","low","medium","high","extreme"])]
    effort: Option<String>,
    #[arg(long, global=true, value_parser=["auto","off","on"])]
    agents: Option<String>,
    #[arg(long, global=true, value_parser=["low","medium","high"])]
    verbosity: Option<String>,
    #[arg(long, global=true, value_parser=["auto","off","on"])]
    web: Option<String>,
    #[arg(long, global = true)]
    project_map_limit: Option<usize>,
    #[arg(long, global = true)]
    tool_output_limit: Option<usize>,
    #[arg(long, global = true)]
    budget: Option<usize>,
    #[arg(long, global = true)]
    passthrough: bool,
    #[arg(long, global = true)]
    dry_run: bool,
    #[arg(long, global = true)]
    trace: bool,
    #[arg(long, global = true)]
    json: bool,
    /// Output is plain text by default; accepted for scripting.
    #[arg(long, global = true)]
    no_color: bool,
    #[arg(long, global = true, default_value = "codex")]
    codex_bin: PathBuf,
    #[arg(long, global = true)]
    model: Option<String>,
    /// Inherit Codex permissions unless explicitly selected. Never bypass sandbox.
    #[arg(long, global=true, value_parser=["read-only","workspace-write"])]
    sandbox: Option<String>,
    /// Explicit JSON CompileInput; cannot be combined with a positional task.
    #[arg(long, global = true)]
    input: Option<PathBuf>,
    #[arg(short = 'C', long = "directory", global = true, default_value = ".")]
    directory: PathBuf,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize project policy and ignored private state without overwriting config.
    Init,
    /// Read-only environment/capability checks; never starts a model run.
    Doctor,
    /// Compile locally. No Codex executable, credentials or network required.
    Plan { prompt: Option<String> },
    /// Compile and run Codex once. '--dry-run' only probes version/help.
    Run { prompt: Option<String> },
    /// Explain the latest recorded run (metadata only).
    Explain,
    /// Bounded local history; unavailable provider fields remain null.
    Usage,
    /// Show effective config or set one project key.
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// Print optional setup guidance; never modifies global Codex config.
    Setup {
        #[arg(value_parser=["codex","agents"])]
        target: String,
    },
    /// Explain safe uninstall steps; does not delete user configuration.
    Uninstall,
    /// Print the verified-installer update procedure (no background network).
    Update,
}
#[derive(Subcommand, Debug)]
enum ConfigAction {
    Set { key: String, value: String },
}

fn main() {
    match execute(Cli::parse()) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("ctxc: {}", storage::redact(&format!("{error:#}")));
            std::process::exit(1);
        }
    }
}

fn execute(cli: Cli) -> Result<i32> {
    let root = cli
        .directory
        .canonicalize()
        .context("project directory does not exist")?;
    if !root.is_dir() {
        bail!("project path is not a directory");
    }
    if matches!(cli.command, Some(Commands::Init)) {
        init(&root, cli.json)?;
        return Ok(0);
    }
    if matches!(cli.command, Some(Commands::Doctor)) {
        return doctor(&cli, &root);
    }
    if matches!(cli.command, Some(Commands::Uninstall)) {
        println!("Remove only the ctxc binary in your chosen bin directory (or run cargo uninstall ctxc).\nInspect .ctxc/state before deleting local history. Global Codex configuration is never owned by ctxc.");
        return Ok(0);
    }
    if matches!(cli.command, Some(Commands::Update)) {
        println!("No automatic update was performed. Download a GitHub release and verify SHA256SUMS, or rerun scripts/install.sh / scripts/install.ps1 with an explicit version. See docs/installation.md.");
        return Ok(0);
    }
    if let Some(Commands::Setup { target }) = &cli.command {
        if target == "agents" {
            println!("Optional AGENTS.md guidance (add manually after review):\nKeep changes scoped. Prefer targeted reads/tests. Preserve explicit constraints. Stop when acceptance is met.");
        } else {
            println!("No global profile is required: ctxc applies supported per-run overrides. Use ctxc doctor to inspect capabilities. Authentication and sandbox settings remain managed by Codex.");
        }
        return Ok(0);
    }
    let (cfg, sources) = config::load(&root)?;
    for warning in cfg.warnings() {
        eprintln!("warning: {}", storage::redact(&warning));
    }
    match &cli.command {
        Some(Commands::Config { action }) => {
            if let Some(ConfigAction::Set { key, value }) = action {
                set_config(&root, key, value)?;
            } else if cli.json {
                println!("{}", storage::redact(&serde_json::to_string_pretty(&cfg)?));
            } else {
                println!("{}", storage::redact(&toml::to_string_pretty(&cfg)?));
                eprintln!("Sources: {:?}", sources);
            }
            return Ok(0);
        }
        Some(Commands::Usage) => {
            let runs = storage::history(&root)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&runs)?);
            } else if runs.is_empty() {
                println!("No recorded runs. Missing usage is unavailable, not zero.");
            } else {
                for r in runs {
                    println!("{} · {:?} · effort={:?} · input={} cached={} output={} reasoning={} · exit={}",r.timestamp_unix_ms,r.mode,r.applied_reasoning,usage_value(&r,|u|u.input_tokens),usage_value(&r,|u|u.cached_input_tokens),usage_value(&r,|u|u.output_tokens),usage_value(&r,|u|u.reasoning_tokens),r.result.as_ref().map(|x|x.exit_status.to_string()).unwrap_or_else(||"unavailable".into()));
                }
            }
            return Ok(0);
        }
        Some(Commands::Explain) => {
            let history = storage::history(&root)?;
            if let Some(run) = history.first() {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(run)?);
                } else {
                    println!("Mode: {:?}\nRequested effort: {:?}\nApplied effort: {:?}\nMap: {} bytes\nEstimated input: {} → {} (approximate)",run.mode,run.requested_reasoning,run.applied_reasoning,run.project_map_bytes,run.estimate.input_before,run.estimate.input_after);
                    for event in &run.trace {
                        println!("{} {}", event.action, event.reason);
                    }
                }
            } else {
                println!("No recorded run. Use ctxc plan --trace to inspect a task without running Codex.");
            }
            return Ok(0);
        }
        _ => {}
    }
    let prompt = match &cli.command {
        Some(Commands::Plan { prompt } | Commands::Run { prompt }) => prompt.as_ref(),
        _ => cli.prompt.as_ref(),
    };
    let mut input = if let Some(path) = &cli.input {
        if prompt.is_some() {
            bail!("choose either a prompt or --input, not both");
        }
        serde_json::from_str::<CompileInput>(&project::read_context_file(path, 4 * 1024 * 1024)?)
            .context("invalid CompileInput JSON")?
    } else {
        let Some(prompt) = prompt else {
            bail!("provide a quoted task, e.g. ctxc plan \"Fix README typo\"; see --help");
        };
        let request = if prompt == "-" {
            let mut text = String::new();
            io::stdin()
                .take(4 * 1024 * 1024 + 1)
                .read_to_string(&mut text)?;
            if text.len() > 4 * 1024 * 1024 {
                bail!("stdin prompt exceeds 4 MiB");
            }
            text
        } else {
            prompt.clone()
        };
        CompileInput {
            version: 1,
            request,
            context: vec![],
            policy: cfg.policy()?,
        }
    };
    apply_flags(&cli, &mut input.policy)?;
    let mode = input
        .policy
        .mode
        .unwrap_or_else(|| ctxc_core::classify(&input.request).0);
    let is_plan = matches!(cli.command, Some(Commands::Plan { .. }));
    let mut map_bytes = 0;
    // No tree walk, Git command, or cache read for CHAT/MICRO/passthrough.
    if matches!(mode, Mode::Code | Mode::Complex)
        && !cli.passthrough
        && input.policy.project_map_max_bytes > 0
    {
        let map = project::build(
            &root,
            input.policy.project_map_max_bytes,
            !is_plan && !cli.dry_run,
        )?;
        map_bytes = map.content.len();
        if map.truncated {
            eprintln!("warning: project map/discovery truncated to safety limits");
        }
        input.context.push(Block {
            id: "project-map".into(),
            kind: BlockKind::ProjectMap,
            source: "repository-metadata-untrusted".into(),
            content: map.content,
            preserve: Preserve::Droppable,
            relevance: 0.3,
            ..Block::default()
        });
    }
    let original = input.request.clone();
    let compiled = ctxc_core::compile(input)?;
    if is_plan {
        if cli.passthrough {
            bail!("--passthrough is a run option, not a compiler plan");
        }
        print_plan(&compiled, cli.json, cli.trace)?;
        return Ok(0);
    }
    let binary = adapter::locate(&cli.codex_bin)?;
    let caps = adapter::detect(&binary)?;
    let invocation = adapter::prepare(
        &binary,
        &caps,
        &compiled,
        &original,
        cli.passthrough,
        cli.model.as_deref(),
        cli.sandbox.as_deref(),
    )?;
    for warning in &invocation.warnings {
        eprintln!("warning: {warning}");
    }
    if cli.dry_run {
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"compiled":compiled,"invocation":invocation,"passthrough":cli.passthrough,"stdin_bytes":invocation.stdin.len(),"preview_note":"prompt passed via stdin, omitted from command preview"})
                )?
            );
        } else {
            print_plan(&compiled, false, cli.trace)?;
            println!("\nCommand preview (not shell syntax): {}\nargs: {:?}\nstdin: <{} bytes; omitted>\nApplied reasoning: {:?}",storage::redact(&binary.display().to_string()),invocation.args,invocation.stdin.len(),invocation.applied_reasoning);
        }
        return Ok(0);
    }
    eprintln!(
        "ctxc · {:?} · reasoning={:?} · agents={} · {}",
        compiled.classification,
        invocation.applied_reasoning,
        if compiled.policy.agents { "on" } else { "off" },
        if cli.passthrough {
            "passthrough"
        } else {
            "context=compact"
        }
    );
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = cancelled.clone();
    ctrlc::set_handler(move || {
        signal.store(true, Ordering::SeqCst);
    })
    .context("install interruption handler")?;
    let mut stdout = io::stdout().lock();
    let result = adapter::run(&invocation, &root, cancelled, cli.json, |text| {
        write!(stdout, "{text}")?;
        if !text.ends_with('\n') {
            writeln!(stdout)?;
        }
        stdout.flush()?;
        Ok(())
    })?;
    if result.malformed_events > 0 {
        eprintln!("warning: malformed structured output forwarded raw; usage parsing unavailable");
    }
    // Persist no request, context, paths, response, arbitrary IDs or free-form metadata.
    let trace = if cfg.trace.enabled {
        compiled
            .trace
            .iter()
            .map(|e| TraceEvent {
                action: e.action.clone(),
                block_id: None,
                reason: e.reason.clone(),
            })
            .collect()
    } else {
        vec![]
    };
    let metadata = storage::RunMetadata {
        schema_version: 1,
        timestamp_unix_ms: storage::now(),
        runtime: "codex".into(),
        runtime_version: Some(caps.version),
        mode: compiled.classification,
        requested_reasoning: compiled.reasoning,
        applied_reasoning: invocation.applied_reasoning,
        estimate: compiled.estimate,
        project_map_bytes: map_bytes,
        trace,
        result: Some(result.clone()),
    };
    if let Err(error) = ensure_private_ignore(&root)
        .and_then(|()| storage::save(&root, &metadata, cfg.trace.retain_runs))
    {
        eprintln!(
            "warning: run finished but history could not be saved: {}",
            storage::redact(&error.to_string())
        );
    }
    if result.exit_status != 0 {
        eprintln!("Codex stopped (exit {}). No automatic retry. Fix the issue and invoke ctxc run again with the original task; prompt bodies are not saved.",result.exit_status);
    }
    Ok(result.exit_status)
}

fn usage_value(
    r: &storage::RunMetadata,
    field: impl FnOnce(&UsageSnapshot) -> Option<u64>,
) -> String {
    r.result
        .as_ref()
        .and_then(|x| field(&x.usage))
        .map(|x| x.to_string())
        .unwrap_or_else(|| "unavailable".into())
}
fn apply_flags(cli: &Cli, p: &mut ExecutionPolicy) -> Result<()> {
    if let Some(s) = &cli.mode {
        p.mode = config::parse_mode(s)?;
    }
    if let Some(s) = &cli.effort {
        p.effort = config::parse_effort(s)?;
    }
    if let Some(s) = &cli.agents {
        p.agents = match s.as_str() {
            "on" => true,
            "off" => false,
            _ => p.agents,
        };
    }
    if let Some(s) = &cli.verbosity {
        p.verbosity = s.clone();
    }
    if let Some(s) = &cli.web {
        p.web = s.clone();
    }
    if let Some(n) = cli.project_map_limit {
        if n > 1024 * 1024 {
            bail!("project map limit must be <= 1 MiB");
        }
        p.project_map_max_bytes = n;
    }
    if let Some(n) = cli.tool_output_limit {
        p.tool_output_token_limit = n;
    }
    if let Some(n) = cli.budget {
        p.context_budget = n;
    }
    ctxc_core::validate_policy(p)?;
    Ok(())
}
fn print_plan(c: &CompiledContext, json: bool, trace: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(c)?);
    } else {
        println!("Mode: {:?} · reasoning={:?}\nEstimated tokens: {} → {} (approximate; includes contract overhead)\n\n{}",c.classification,c.reasoning,c.estimate.input_before,c.estimate.input_after,c.contract);
        if trace {
            for t in &c.trace {
                println!("{} {}", t.action, t.reason);
            }
        }
    }
    if c.budget_exceeded {
        eprintln!("warning: protected content exceeds context budget; retained intact");
    }
    Ok(())
}
fn ensure_private_ignore(root: &Path) -> Result<()> {
    let path = root.join(".ctxc/.gitignore");
    storage::ensure_safe(root, &path)?;
    let mut text = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    for entry in ["/state/", "/project-map.json"] {
        if !text.lines().any(|l| l == entry) {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(entry);
            text.push('\n');
        }
    }
    storage::atomic_write(root, &path, text.as_bytes())
}
fn init(root: &Path, json: bool) -> Result<()> {
    let path = root.join(".ctxc/config.toml");
    storage::ensure_safe(root, &path)?;
    if !path.exists() {
        storage::atomic_write(
            root,
            &path,
            toml::to_string_pretty(&config::Config::default())?.as_bytes(),
        )?;
    }
    ensure_private_ignore(root)?;
    let ignore = root.join(".gitignore");
    storage::ensure_safe(root, &ignore)?;
    let mut text = if ignore.exists() {
        fs::read_to_string(&ignore)?
    } else {
        String::new()
    };
    for entry in [".ctxc/state/", ".ctxc/project-map.json"] {
        if !text.lines().any(|l| l == entry) {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(entry);
            text.push('\n');
        }
    }
    storage::atomic_write(root, &ignore, text.as_bytes())?;
    let (cfg, _) = config::load(root)?;
    let map = project::build(root, cfg.context.project_map_max_bytes, true)?;
    storage::atomic_write(
        root,
        &root.join(".ctxc/project-map.json"),
        &serde_json::to_vec_pretty(&map)?,
    )?;
    let state = root.join(".ctxc/state");
    storage::ensure_safe(root, &state)?;
    fs::create_dir_all(state)?;
    if json {
        println!("{{\"initialized\":true,\"global_config_modified\":false}}");
    } else {
        println!("Initialized .ctxc. Existing config preserved; private state ignored. No global Codex config changed.");
    }
    Ok(())
}
fn doctor(cli: &Cli, root: &Path) -> Result<i32> {
    let runtime = adapter::locate(&cli.codex_bin).and_then(|p| adapter::detect(&p));
    let cfg = config::load(root);
    let git = std::process::Command::new("git")
        .args(["status", "--short", "--untracked-files=no"])
        .current_dir(root)
        .output();
    let value = serde_json::json!({"ctxc_version":env!("CARGO_PKG_VERSION"),"codex":runtime.as_ref().ok(),"codex_error":runtime.as_ref().err().map(|e|storage::redact(&e.to_string())),"git_available":git.is_ok(),"is_git_repository":git.as_ref().is_ok_and(|o|o.status.success()),"tracked_changes":git.as_ref().ok().filter(|o|o.status.success()).map(|o|String::from_utf8_lossy(&o.stdout).lines().count()),"config_sources":cfg.as_ref().ok().map(|(_,s)|s),"config_error":cfg.as_ref().err().map(|e|storage::redact(&e.to_string())),"model_calls":0});
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(if runtime.is_ok() && cfg.is_ok() { 0 } else { 1 })
}
fn set_config(root: &Path, key: &str, value: &str) -> Result<()> {
    let keys: Vec<&str> = key.split('.').collect();
    let defaults = toml::Value::try_from(config::Config::default())?;
    let mut current = &defaults;
    for k in &keys {
        current = current
            .get(k)
            .with_context(|| format!("unknown config key {key}"))?;
    }
    if current.is_table() {
        bail!("set a leaf key, not an entire table");
    }
    let path = root.join(".ctxc/config.toml");
    storage::ensure_safe(root, &path)?;
    let text = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        toml::to_string_pretty(&config::Config::default())?
    };
    let mut doc = text.parse::<toml_edit::DocumentMut>()?;
    let parsed = format!("value = {value}")
        .parse::<toml_edit::DocumentMut>()
        .or_else(|_| {
            format!("value = {}", serde_json::to_string(value).unwrap())
                .parse::<toml_edit::DocumentMut>()
        })?;
    let mut item = doc.as_item_mut();
    for k in keys {
        item = &mut item[k];
    }
    *item = parsed["value"].clone();
    let candidate: config::Config = toml::from_str(&doc.to_string())?;
    candidate.policy()?;
    storage::atomic_write(root, &path, doc.to_string().as_bytes())?;
    println!("Updated {key} in project config.");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shorthand_and_commands() {
        assert_eq!(
            Cli::try_parse_from(["ctxc", "hello"])
                .unwrap()
                .prompt
                .as_deref(),
            Some("hello")
        );
        assert!(matches!(
            Cli::try_parse_from(["ctxc", "run", "hello", "--dry-run"])
                .unwrap()
                .command,
            Some(Commands::Run { .. })
        ));
    }
    #[test]
    fn init_idempotent() {
        let d = tempfile::tempdir().unwrap();
        init(d.path(), false).unwrap();
        let first = fs::read(d.path().join(".gitignore")).unwrap();
        init(d.path(), false).unwrap();
        assert_eq!(first, fs::read(d.path().join(".gitignore")).unwrap());
    }
    #[test]
    fn config_preserves_comments() {
        let d = tempfile::tempdir().unwrap();
        init(d.path(), false).unwrap();
        let p = d.path().join(".ctxc/config.toml");
        let text = fs::read_to_string(&p).unwrap();
        fs::write(&p, format!("# user comment\n{text}")).unwrap();
        set_config(d.path(), "context.budget", "9000").unwrap();
        assert!(fs::read_to_string(p).unwrap().starts_with("# user comment"));
    }
}
