#![allow(dead_code, unused_imports)]

use clap::{Parser, Subcommand};
use colored::*;

mod agent;
mod benchmark;
mod budget;
mod config;
mod context;
mod memory;
mod policy;
mod providers;
mod repl;
mod routing;
mod security;
mod tasks;
mod tools;
mod usage;

#[derive(Parser)]
#[command(name = "ind")]
#[command(author = "Deepak <https://github.com/Deepak-ai-93/deepak-Ind>")]
#[command(version = "0.1.0")]
#[command(about = "A token-efficient, provider-neutral terminal coding agent in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Task prompt to execute interactively
    #[arg(trailing_var_arg = true)]
    task: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive Pi AI coding REPL session
    Chat,
    /// Start an interactive Pi AI coding REPL session (alias for chat)
    Repl,
    /// Execute a task with chunked auto-approved tool execution
    Run {
        /// Task description
        task: String,
    },
    /// Preview bounded task chunks without running them
    Plan {
        /// Task description
        task: String,
    },
    /// Preview token-budgeted repository context
    Context {
        /// Task description
        task: String,
    },
    /// Show the selected cheap/strong model tier
    Route {
        /// Task description
        task: String,
    },
    /// Estimate tokens, turns, savings, and cost
    Budget {
        /// Task description
        task: String,
    },
    /// Show configured provider capabilities
    Providers,
    /// Detect local Ollama and LM Studio runtimes
    Discover,
    /// Show local tokens, cost, latency, and savings
    Usage,
    /// Manage project memory (MEMORY.md)
    Memory {
        #[command(subcommand)]
        sub: Option<MemorySubcommands>,
    },
    /// Show resumable session state
    Resume,
    /// Run the context savings benchmark
    Benchmark,
    /// Record a reproducible benchmark ranking
    Leaderboard,
    /// Check environment and likely secrets
    Doctor,
    /// Show resolved project configuration
    Config,
    /// Show project team policy
    Policy,
}

#[derive(Subcommand)]
enum MemorySubcommands {
    /// Append a memory note
    Add {
        /// Memory category (e.g. decisions, conventions)
        entry_type: String,
        /// Note text
        #[arg(trailing_var_arg = true)]
        note: Vec<String>,
    },
    /// Sync encrypted memory with remote server
    Sync {
        /// push or pull
        action: String,
        /// Optional sync URL
        url: Option<String>,
    },
}

fn print_config() {
    let cfg = config::load_config(None);
    println!("{}", "╭── ⚙️ IND Configuration (Deepak Bagada Edition) ─────────────────────────╮".bright_cyan().bold());
    println!("│  Project Root:     {:<50} │", cfg.project_root.display().to_string().bright_yellow());
    println!("│  Provider:         {:<50} │", cfg.provider.bright_green().bold());
    println!(
        "│  Model:            {:<50} │",
        if cfg.model.is_empty() {
            "not selected".dimmed().to_string()
        } else {
            cfg.model.bright_white().bold().to_string()
        }
    );
    println!(
        "│  Cheap Tier Model: {:<50} │",
        if cfg.cheap_model.is_empty() {
            "not selected".dimmed().to_string()
        } else {
            cfg.cheap_model.bright_white().to_string()
        }
    );
    println!(
        "│  Strong Tier Model:{:<50} │",
        if cfg.strong_model.is_empty() {
            "not selected".dimmed().to_string()
        } else {
            cfg.strong_model.bright_white().to_string()
        }
    );
    println!("│  Routing Mode:     {:<50} │", format!("{:?}", cfg.routing).bright_cyan());
    println!(
        "│  Base Endpoint:    {:<50} │",
        cfg.base_url.as_deref().unwrap_or("provider default").dimmed()
    );
    println!("│  Approval Policy:  {:<50} │", format!("{:?}", cfg.approval).bright_magenta());
    println!("│  Input Budget:     {:<50} │", format!("{} tokens", cfg.max_input_tokens).bright_green());
    println!("│  Output Budget:    {:<50} │", format!("{} tokens", cfg.max_output_tokens).bright_green());
    println!("│  Max Tool Turns:   {:<50} │", cfg.max_tool_turns.to_string().bright_yellow());
    println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
}

fn print_policy() {
    let cfg = config::load_config(None);
    match policy::load_policy(&cfg.project_root) {
        Ok(p) => {
            println!("{}", "╭── 🛡️ IND Security & Execution Policy ──────────────────────────────────────╮".bright_cyan().bold());
            println!(
                "│  Policy Path:     {:<50} │",
                policy::policy_path(&cfg.project_root).display().to_string().bright_yellow()
            );
            println!(
                "│  Approval Mode:   {:<50} │",
                p.approval
                    .map(|a| format!("{a:?}"))
                    .unwrap_or_else(|| "config default".to_string())
                    .bright_magenta()
            );
            println!(
                "│  Allowed Providers:{:<49} │",
                p.allowed_providers
                    .as_ref()
                    .map(|list| list.join(", "))
                    .unwrap_or_else(|| "all configured providers".to_string())
                    .bright_green()
            );
            println!(
                "│  Command Allowlist:{:<49} │",
                p.allowed_commands
                    .as_ref()
                    .map(|list| list.join(" | "))
                    .unwrap_or_else(|| "not restricted".to_string())
                    .bright_cyan()
            );
            println!(
                "│  Command Denylist: {:<49} │",
                p.denied_commands
                    .as_ref()
                    .map(|list| list.join(" | "))
                    .unwrap_or_else(|| "safety defaults only".to_string())
                    .bright_red()
            );
            println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
        }
        Err(e) => {
            eprintln!("{} {e}", "✖ IND policy error:".red().bold());
            std::process::exit(1);
        }
    }
}

fn print_plan(task: &str) {
    match tasks::planner::create_task_plan(task, &[]) {
        Ok(plan) => {
            println!("{}", format!("╭── 📋 Plan ID: {} ─────────────────────────────────────────────────────╮", plan.id).bright_cyan().bold());
            for chunk in &plan.chunks {
                println!(
                    "│  {}. {} — {}",
                    chunk.sequence.to_string().bright_yellow().bold(),
                    chunk.title.bright_white().bold(),
                    chunk.goal.dimmed()
                );
            }
            println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
        }
        Err(e) => {
            eprintln!("{} {e}", "✖ IND plan error:".red().bold());
            std::process::exit(1);
        }
    }
}

fn print_context(task: &str) -> Result<(), String> {
    let cfg = config::load_config(None);
    let selection = context::inspect_and_select(&cfg.project_root, task, cfg.max_input_tokens)?;
    println!("{}", "╭── 📂 Token-Budget Context Selection ──────────────────────────────────────╮".bright_cyan().bold());
    println!("│  Task:     {}", selection.task.bright_yellow().bold());
    println!(
        "│  Selected: {} files / ~{} tokens",
        selection.selected.len().to_string().bright_green().bold(),
        selection.estimated_tokens.to_string().bright_cyan().bold()
    );
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    for file in &selection.selected {
        println!(
            "│  + {:<40} (~{} tokens; {})",
            file.relative_path.bright_white().bold(),
            file.estimated_tokens.to_string().bright_yellow(),
            file.reason.dimmed()
        );
    }
    if !selection.omitted.is_empty() {
        println!("├─────────────────────────────────────────────────────────────────────────────┤");
        println!(
            "│  Omitted: {} files (budget {} tokens)",
            selection.omitted.len().to_string().bright_red(),
            selection.budget_tokens.to_string().dimmed()
        );
    }
    println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
    Ok(())
}

fn print_route(task: &str) {
    let cfg = config::load_config(None);
    let decision = routing::route_task(task, &cfg);
    println!("{}", "╭── 🔀 Task Tier Model Routing ──────────────────────────────────────────────╮".bright_cyan().bold());
    println!("│  Task Kind: {:<48} │", decision.kind.as_str().bright_yellow());
    println!("│  Model Tier: {:<47} │", decision.tier.to_string().bright_green().bold());
    println!(
        "│  Model Name: {:<47} │",
        if decision.model.is_empty() {
            "not selected".dimmed().to_string()
        } else {
            decision.model.bright_white().bold().to_string()
        }
    );
    println!("│  Reason:     {:<48} │", decision.reason.dimmed());
    println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
}

fn print_budget(task: &str) -> Result<(), String> {
    let cfg = config::load_config(None);
    let selection = context::inspect_and_select(&cfg.project_root, task, cfg.max_input_tokens)?;
    let plan = tasks::planner::create_task_plan(task, &[])?;
    let route = routing::route_task(task, &cfg);
    let budget = budget::create_budget_plan(&cfg, &selection, task, plan.chunks.len(), &route);
    println!("{}", "╭── 💰 Token Budget & Cost Estimate ────────────────────────────────────────╮".bright_cyan().bold());
    println!("│  Task:            {:<48} │", budget.task.bright_yellow());
    println!(
        "│  Provider/Model:  {:<48} │",
        format!(
            "{}/{}",
            budget.provider,
            if budget.model.is_empty() {
                "not selected"
            } else {
                &budget.model
            }
        ).bright_green().bold()
    );
    println!("│  Model Tier:      {:<48} │", budget.tier.to_string().bright_cyan());
    println!(
        "│  Input Allowance: {:<48} │",
        format!("~{} tokens (context {})", budget.estimated_input_tokens, budget.context_tokens).bright_white()
    );
    println!("│  Output Allowance:{:<48} │", format!("~{} tokens", budget.estimated_output_tokens).bright_white());
    println!("│  Tool Turns:      {:<48} │", budget.estimated_tool_turns.to_string().bright_yellow());
    println!("│  Total Allowance: {:<48} │", format!("~{} tokens", budget.estimated_total_tokens).bright_green().bold());
    println!("│  Baseline Input:  {:<48} │", format!("~{} tokens", budget.baseline_input_tokens).dimmed());
    println!(
        "│  Est. Savings:    {:<48} │",
        format!("{:.1}%", budget.estimated_savings_percent).bright_green().bold()
    );
    println!("│  Estimated Cost:  ${:<47.6} │", budget.estimated_cost);
    if !budget.warnings.is_empty() {
        println!("├─────────────────────────────────────────────────────────────────────────────┤");
        for warning in &budget.warnings {
            println!("│  ⚠ Warning: {:<47} │", warning.bright_yellow());
        }
    }
    println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
    Ok(())
}

fn run_task(task: &str) -> Result<(), String> {
    let cfg = config::load_config(None);
    let mut plan = tasks::planner::create_task_plan(task, &[])?;
    let selection = context::inspect_and_select(&cfg.project_root, task, cfg.max_input_tokens)?;
    plan.context_files = selection
        .selected
        .iter()
        .map(|f| f.relative_path.clone())
        .collect();

    println!("{} {}", "Executing task:".bold().cyan(), plan.task);
    println!(
        "{} {} files in token budget.",
        "Loaded".bold().cyan(),
        plan.context_files.len()
    );

    let approved: Vec<String> = plan.chunks.iter().map(|c| c.id.clone()).collect();
    let policy = policy::load_policy(&cfg.project_root).unwrap_or_default();
    let mut failed = false;

    tasks::runner::run_task_plan(
        &mut plan,
        &std::collections::HashMap::new(),
        &tasks::runner::RunPlanOptions {
            project_root: &cfg.project_root,
            approved_chunks: &approved,
            policy: Some(policy),
        },
        &mut |event| {
            let line = match &event {
                tasks::runner::TaskRunEvent::ApprovalRequired { .. } => {
                    format!("{} {}", "[Approval]".yellow().bold(), event.message())
                }
                tasks::runner::TaskRunEvent::ChunkStart { .. } => {
                    format!("{} {}", "[Start]".cyan().bold(), event.message())
                }
                tasks::runner::TaskRunEvent::ChunkPassed { .. } => {
                    format!("{} {}", "[Passed]".green().bold(), event.message())
                }
                tasks::runner::TaskRunEvent::ChunkFailed { .. } => {
                    failed = true;
                    format!("{} {}", "[Failed]".red().bold(), event.message())
                }
                tasks::runner::TaskRunEvent::PlanComplete { .. } => {
                    format!("{} {}", "[Complete]".green().bold(), event.message())
                }
                _ => event.message().to_string(),
            };
            println!("  {line}");
        },
    )?;

    if failed {
        std::process::exit(1);
    }
    Ok(())
}

fn print_usage() {
    println!("{}", "╭── 📊 Local Token Usage & Savings Ledger (SQLite) ────────────────────────╮".bright_cyan().bold());
    println!("│  Engine:           Native Rust (Zero External Runtime)                   │");
    println!("│  Memory Footprint: < 10MB                                                │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    match usage::UsageLedger::init(&config::load_config(None).project_root) {
        Ok(ledger) => match ledger.summary() {
            Ok((prompt, completion, cost)) => {
                println!("│  Prompt Tokens:     {:<48} │", prompt.to_string().bright_yellow());
                println!("│  Completion Tokens: {:<48} │", completion.to_string().bright_yellow());
                println!("│  Total Tokens:      {:<48} │", (prompt + completion).to_string().bright_green().bold());
                println!("│  Estimated Cost:    ${:<47.4} │", cost);
            }
            Err(e) => eprintln!("│  {} {:<48} │", "✖ Ledger error:".red(), e),
        },
        Err(e) => eprintln!("│  {} {:<48} │", "✖ Ledger error:".red(), e),
    }
    println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
}

fn print_memory(sub: Option<MemorySubcommands>) -> Result<(), String> {
    let cfg = config::load_config(None);
    let mut mem = memory::MemoryManager::new(&cfg.project_root);
    match sub {
        Some(MemorySubcommands::Add { entry_type, note }) => {
            mem.append(&entry_type, &note.join(" "))
                .map_err(|e| format!("Failed to append memory: {e}"))?;
            println!("{} Added note to MEMORY.md", "[Success]".green());
        }
        Some(MemorySubcommands::Sync { action, url }) => {
            let url = url
                .or_else(|| std::env::var("IND_SYNC_URL").ok())
                .filter(|s| !s.trim().is_empty());
            let secret = std::env::var("IND_SYNC_KEY")
                .ok()
                .filter(|s| !s.trim().is_empty());
            match (url, secret) {
                (Some(_url), Some(_secret)) => {
                    eprintln!(
                        "{} Encrypted sync is configured; Phase 1 builds local encrypted memory only.",
                        "[Sync]".yellow().bold()
                    );
                    println!("Syncing memory ({action})...");
                }
                _ => return Err("Memory sync requires IND_SYNC_URL and IND_SYNC_KEY.".to_string()),
            }
        }
        None => {
            println!("{}", "Viewing Project Memory:".bold().cyan());
            let content = mem
                .read()
                .map_err(|e| format!("Failed to read memory: {e}"))?;
            if content.trim().is_empty() {
                println!("No MEMORY.md found at {}", cfg.project_root.display());
            } else {
                println!("{content}");
            }
        }
    }
    Ok(())
}

async fn print_discover() -> Result<(), String> {
    let summaries = providers::local_runtime_summary()
        .await
        .map_err(|e| format!("Discovery error: {e}"))?;
    for line in summaries {
        println!("{line}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cfg = config::load_config(None);

    let result: Result<(), String> = match cli.command {
        Some(Commands::Chat) | Some(Commands::Repl) => {
            repl::start_repl(cfg).await.map_err(|e| e.to_string())?;
            Ok(())
        }
        Some(Commands::Run { task }) => run_task(&task),
        Some(Commands::Plan { task }) => {
            print_plan(&task);
            Ok(())
        }
        Some(Commands::Context { task }) => print_context(&task),
        Some(Commands::Route { task }) => {
            print_route(&task);
            Ok(())
        }
        Some(Commands::Budget { task }) => print_budget(&task),
        Some(Commands::Providers) => {
            println!("{}", "Configured Provider Settings:".bold().green());
            match providers::provider_summary() {
                Ok(lines) => {
                    for line in lines {
                        println!("  {line}");
                    }
                }
                Err(e) => return Err(e.into()),
            }
            Ok(())
        }
        Some(Commands::Discover) => {
            println!("{}", "Probing local runtimes...".bold().yellow());
            print_discover().await
        }
        Some(Commands::Usage) => {
            print_usage();
            Ok(())
        }
        Some(Commands::Memory { sub }) => print_memory(sub),
        Some(Commands::Resume) => {
            println!("{}", "Checking resumable state...".bold().cyan());
            match memory::MemoryManager::new(&cfg.project_root).resume_state() {
                Ok(Some(state)) => println!("{state}"),
                _ => println!("No saved IND session to resume."),
            }
            Ok(())
        }
        Some(Commands::Benchmark) => {
            println!("{}", "Running context savings benchmark...".bold().cyan());
            let entry = benchmark::run_all(&cfg.project_root)?;
            println!(
                "  Fixtures: {} run, {} passed",
                entry.fixtures_run, entry.fixtures_passed
            );
            println!();
            for result in &entry.results {
                benchmark::print_result(result);
            }
            println!();
            println!(
                "  Median savings: {:.1}%  |  Mean savings: {:.1}%",
                entry.median_savings_percent, entry.mean_savings_percent
            );
            println!("  Verdict: {}", entry.verdict);
            Ok(())
        }
        Some(Commands::Leaderboard) => {
            println!(
                "{}",
                "Running reproducible benchmark leaderboard..."
                    .bold()
                    .cyan()
            );
            let entry = benchmark::run_all(&cfg.project_root)?;
            let report_path = benchmark::write_leaderboard_report(&cfg.project_root, &entry)?;
            println!("  Leaderboard generated: {}", report_path.display());
            println!(
                "  Median savings: {:.1}% ({} fixtures run)",
                entry.median_savings_percent, entry.fixtures_run
            );
            println!("  Verdict: {}", entry.verdict);
            Ok(())
        }
        Some(Commands::Doctor) => {
            println!("{}", "╭── 🩺 IND Doctor Diagnostics (Deepak Bagada Edition) ──────────────────╮".bright_cyan().bold());
            println!();

            // Section 1: Configuration summary.
            println!("{}", "── ⚙️ Configuration ──".bright_yellow().bold());
            println!("  Project Root:      {}", cfg.project_root.display().to_string().cyan());
            println!("  Provider:          {}", cfg.provider.bright_green());
            println!("  Approval Mode:     {:?}", cfg.approval);
            println!("  Max input tokens:  {}", cfg.max_input_tokens);
            println!("  Max output tokens: {}", cfg.max_output_tokens);
            println!("  Max tool turns:    {}", cfg.max_tool_turns);
            println!();

            // Section 2: Toolchain checks.
            println!("{}", "── 🛠️ Toolchain ──".bright_yellow().bold());
            for f in security::check_toolchain() {
                println!("  [{}] {}", f.severity.label(), f.message);
            }
            println!();

            // Section 3: Provider API keys.
            println!("{}", "── 🔑 Provider Keys ──".bright_yellow().bold());
            for f in security::check_provider_keys() {
                println!("  [{}] {}", f.severity.label(), f.message);
            }
            println!();

            // Section 4: Security scanner.
            println!("{}", "── 🛡️ Security Scanner ──".bright_yellow().bold());
            let report = security::scan_project(&cfg.project_root);
            for f in &report.findings {
                let file_suffix = f
                    .file
                    .as_ref()
                    .map(|p| format!(" ({})", p))
                    .unwrap_or_default();
                println!(
                    "  [{}] [{}] {}{}",
                    f.severity.label(),
                    f.category.bold(),
                    f.message,
                    file_suffix
                );
            }
            println!();

            // Summary.
            let (pass, info, warn, crit) = report.counts();
            println!("{}", "── 📊 Summary ──".bright_yellow().bold());
            println!(
                "  {} pass, {} info, {} warnings, {} critical",
                pass.to_string().bright_green().bold(),
                info.to_string().bright_blue().bold(),
                warn.to_string().bright_yellow().bold(),
                crit.to_string().bright_red().bold()
            );
            if report.is_healthy() {
                println!(
                    "  {}",
                    "✔ Project is healthy — no critical issues found."
                        .bright_green()
                        .bold()
                );
            } else {
                println!(
                    "  {}",
                    "✖ Critical issues detected — review findings above."
                        .bright_red()
                        .bold()
                );
            }
            println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
            Ok(())
        }
        Some(Commands::Config) => {
            print_config();
            Ok(())
        }
        Some(Commands::Policy) => {
            print_policy();
            Ok(())
        }
        None => {
            let task_str = cli.task.join(" ");
            if task_str.trim().is_empty() {
                repl::start_repl(cfg).await.map_err(|e| e.to_string())?;
                Ok(())
            } else {
                run_task(&task_str)
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{} {e}", "IND error:".red().bold());
        std::process::exit(1);
    }
    Ok(())
}
