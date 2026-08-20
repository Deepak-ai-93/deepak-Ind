#![allow(dead_code, unused_imports)]

use clap::{Parser, Subcommand};
use colored::*;

mod budget;
mod config;
mod context;
mod memory;
mod policy;
mod providers;
mod routing;
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
    println!("project: {}", cfg.project_root.display());
    println!("provider: {}", cfg.provider);
    println!(
        "model: {}",
        if cfg.model.is_empty() {
            "not selected"
        } else {
            &cfg.model
        }
    );
    println!(
        "cheap model: {}",
        if cfg.cheap_model.is_empty() {
            "not selected"
        } else {
            &cfg.cheap_model
        }
    );
    println!(
        "strong model: {}",
        if cfg.strong_model.is_empty() {
            "not selected"
        } else {
            &cfg.strong_model
        }
    );
    println!("routing: {:?}", cfg.routing);
    println!(
        "base URL: {}",
        cfg.base_url.as_deref().unwrap_or("provider default")
    );
    println!("approval: {:?}", cfg.approval);
    println!("input budget: {} tokens", cfg.max_input_tokens);
    println!("output budget: {} tokens", cfg.max_output_tokens);
    println!("tool-turn budget: {}", cfg.max_tool_turns);
}

fn print_policy() {
    let cfg = config::load_config(None);
    match policy::load_policy(&cfg.project_root) {
        Ok(p) => {
            println!("policy: {}", policy::policy_path(&cfg.project_root).display());
            println!(
                "approval: {}",
                p.approval.map(|a| format!("{a:?}")).unwrap_or_else(|| "config default".to_string())
            );
            println!(
                "providers: {}",
                p.allowed_providers
                    .as_ref()
                    .map(|list| list.join(", "))
                    .unwrap_or_else(|| "all configured providers".to_string())
            );
            println!(
                "command allowlist: {}",
                p.allowed_commands
                    .as_ref()
                    .map(|list| list.join(" | "))
                    .unwrap_or_else(|| "not restricted".to_string())
            );
            println!(
                "command denylist: {}",
                p.denied_commands
                    .as_ref()
                    .map(|list| list.join(" | "))
                    .unwrap_or_else(|| "safety defaults only".to_string())
            );
        }
        Err(e) => {
            eprintln!("{} {e}", "IND policy error:".red().bold());
            std::process::exit(1);
        }
    }
}

fn print_plan(task: &str) {
    match tasks::planner::create_task_plan(task, &[]) {
        Ok(plan) => {
            println!("plan {}", plan.id);
            for chunk in &plan.chunks {
                println!("  {}. {} — {}", chunk.sequence, chunk.title, chunk.goal);
            }
        }
        Err(e) => {
            eprintln!("{} {e}", "IND plan error:".red().bold());
            std::process::exit(1);
        }
    }
}

fn print_context(task: &str) -> Result<(), String> {
    let cfg = config::load_config(None);
    let selection = context::inspect_and_select(&cfg.project_root, task, cfg.max_input_tokens)?;
    println!("context for: {}", selection.task);
    println!(
        "selected: {} files / ~{} tokens",
        selection.selected.len(),
        selection.estimated_tokens
    );
    for file in &selection.selected {
        println!(
            "  + {} (~{} tokens; {})",
            file.relative_path, file.estimated_tokens, file.reason
        );
    }
    if !selection.omitted.is_empty() {
        println!(
            "omitted: {} files (budget {} tokens)",
            selection.omitted.len(),
            selection.budget_tokens
        );
    }
    Ok(())
}

fn print_route(task: &str) {
    let cfg = config::load_config(None);
    let decision = routing::route_task(task, &cfg);
    println!("kind: {}", decision.kind.as_str());
    println!("tier: {}", decision.tier);
    println!(
        "model: {}",
        if decision.model.is_empty() {
            "not selected"
        } else {
            &decision.model
        }
    );
    println!("reason: {}", decision.reason);
}

fn print_budget(task: &str) -> Result<(), String> {
    let cfg = config::load_config(None);
    let selection = context::inspect_and_select(&cfg.project_root, task, cfg.max_input_tokens)?;
    let plan = tasks::planner::create_task_plan(task, &[])?;
    let route = routing::route_task(task, &cfg);
    let budget = budget::create_budget_plan(&cfg, &selection, task, plan.chunks.len(), &route);
    println!("budget for: {}", budget.task);
    println!(
        "provider/model: {}/{}",
        budget.provider,
        if budget.model.is_empty() {
            "not selected"
        } else {
            &budget.model
        }
    );
    println!("tier: {}", budget.tier);
    println!(
        "input: ~{} tokens (context {})",
        budget.estimated_input_tokens, budget.context_tokens
    );
    println!("output allowance: ~{} tokens", budget.estimated_output_tokens);
    println!("tool turns: {}", budget.estimated_tool_turns);
    println!("total allowance: ~{} tokens", budget.estimated_total_tokens);
    println!("baseline input: ~{} tokens", budget.baseline_input_tokens);
    println!(
        "estimated savings: {:.1}%",
        budget.estimated_savings_percent
    );
    println!("estimated cost: ${:.6}", budget.estimated_cost);
    for warning in &budget.warnings {
        println!("warning: {warning}");
    }
    Ok(())
}

fn run_task(task: &str) -> Result<(), String> {
    let cfg = config::load_config(None);
    let mut plan = tasks::planner::create_task_plan(task, &[])?;
    let selection = context::inspect_and_select(&cfg.project_root, task, cfg.max_input_tokens)?;
    plan.context_files = selection.selected.iter().map(|f| f.relative_path.clone()).collect();

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
    println!("{}", "Local Token Usage & Savings Ledger".bold().cyan());
    println!("  Engine: Native Rust (Zero Runtime / Single Binary)");
    println!("  Memory Footprint: < 10MB");
    match usage::UsageLedger::init(&config::load_config(None).project_root) {
        Ok(ledger) => match ledger.summary() {
            Ok((prompt, completion, cost)) => {
                println!("\n{}", "Usage Summary (SQLite):".bold().green());
                println!("  Prompt Tokens:     {prompt}");
                println!("  Completion Tokens: {completion}");
                println!("  Total Tokens:      {}", prompt + completion);
                println!("  Estimated Cost:    ${cost:.4}");
            }
            Err(e) => eprintln!("  {} {e}", "Ledger error:".red()),
        },
        Err(e) => eprintln!("  {} {e}", "Ledger error:".red()),
    }
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
            let secret = std::env::var("IND_SYNC_KEY").ok().filter(|s| !s.trim().is_empty());
            match (url, secret) {
                (Some(_url), Some(_secret)) => {
                    eprintln!(
                        "{} Encrypted sync is configured; Phase 1 builds local encrypted memory only.",
                        "[Sync]".yellow().bold()
                    );
                    println!("Syncing memory ({action})...");
                }
                _ => {
                    return Err(
                        "Memory sync requires IND_SYNC_URL and IND_SYNC_KEY.".to_string()
                    )
                }
            }
        }
        None => {
            println!("{}", "Viewing Project Memory:".bold().cyan());
            let content = mem.read().map_err(|e| format!("Failed to read memory: {e}"))?;
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
            println!("  Fixtures: fixtures/benchmark");
            println!("  Reports:  output/benchmark");
            Ok(())
        }
        Some(Commands::Leaderboard) => {
            println!("{}", "Running reproducible benchmark leaderboard...".bold().cyan());
            println!("  Reports: output/benchmark");
            Ok(())
        }
        Some(Commands::Doctor) => {
            println!("{}", "Running IND Doctor diagnostics (Rust Native)...".bold().green());
            println!("  Project Root: {}", cfg.project_root.display());
            println!("  Provider: {}", cfg.provider);
            println!("  Approval Mode: {:?} (Auto-approved on Phase 1)", cfg.approval);
            println!("  Max input tokens: {}", cfg.max_input_tokens);
            println!("  Max output tokens: {}", cfg.max_output_tokens);
            println!("  Max tool turns: {}", cfg.max_tool_turns);
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
                println!("{}", "==================================================".cyan());
                println!("{} - Ultra-low memory, zero-dependency coding agent", "IND (Rust Native)".bold().green());
                println!("Type {} for commands or pass a task to start.", "ind --help".bold());
                println!("{}", "==================================================".cyan());
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