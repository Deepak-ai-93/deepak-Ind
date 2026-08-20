use std::fs;
use std::io::{self, Write};
use std::path::Path;

use colored::*;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::agent::AgentSession;
use crate::config::{IndConfig, load_config};
use crate::memory::MemoryManager;
use crate::providers::create_configured_provider;
use crate::security;
use crate::tasks::planner::create_task_plan;
use crate::usage::UsageLedger;

pub async fn start_repl(mut cfg: IndConfig) -> Result<(), Box<dyn std::error::Error>> {
    print_welcome_banner(&cfg);

    let ind_dir = cfg.project_root.join(".ind");
    let _ = fs::create_dir_all(&ind_dir);
    let history_file = ind_dir.join("repl_history.txt");

    let mut rl = DefaultEditor::new()?;
    if history_file.exists() {
        let _ = rl.load_history(&history_file);
    }

    let mut session = AgentSession::new(cfg.clone());

    loop {
        let prompt_str = format!("{} ", "⚡ deepak-ind ❯".bright_cyan().bold());
        let readline = rl.readline(&prompt_str);

        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(trimmed);

                // Handle slash commands
                if trimmed.starts_with('/') {
                    let should_exit = handle_slash_command(trimmed, &mut cfg, &mut session)?;
                    if should_exit {
                        break;
                    }
                    continue;
                }

                // Execute autonomous AI agent turn
                match create_configured_provider() {
                    Ok(provider) => {
                        println!();
                        println!("{}", "╭─ 🤖 deepak-ai ─────────────────────────────────────────────────────────────".bright_magenta().bold());
                        print!("│ ");
                        let _ = io::stdout().flush();

                        let res = session
                            .run_turn(provider.as_ref(), trimmed, |token| {
                                if token.contains('\n') {
                                    let replaced = token.replace('\n', "\n│ ");
                                    print!("{replaced}");
                                } else {
                                    print!("{token}");
                                }
                                let _ = io::stdout().flush();
                            })
                            .await;

                        println!();
                        println!("{}", "╰─────────────────────────────────────────────────────────────────────────────".bright_magenta().bold());
                        println!();
                        if let Err(e) = res {
                            eprintln!("{} {e}", "✖ Agent Error:".red().bold());
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {e}", "✖ Provider Config Error:".red().bold());
                        eprintln!(
                            "💡 Tip: Set {} or run {} to switch model/provider.",
                            "OPENAI_API_KEY".bold().yellow(),
                            "/model".bright_cyan()
                        );
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C (Type /exit or Ctrl+D to quit)".yellow());
            }
            Err(ReadlineError::Eof) => {
                println!(
                    "{}",
                    "Goodbye! Thanks for coding with Deepak Bagada IND AI."
                        .bright_cyan()
                        .bold()
                );
                break;
            }
            Err(err) => {
                eprintln!("{} {err:?}", "Readline Error:".red());
                break;
            }
        }
    }

    let _ = rl.save_history(&history_file);
    Ok(())
}

fn handle_slash_command(
    cmd: &str,
    cfg: &mut IndConfig,
    session: &mut AgentSession,
) -> Result<bool, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let root_cmd = parts.first().copied().unwrap_or("");

    match root_cmd {
        "/help" | "/h" => {
            println!(
                "{}",
                "╭── ⚡ IND Slash Commands Cheatsheet (Deepak Bagada Edition) ──────────────────╮"
                    .bright_cyan()
                    .bold()
            );
            println!(
                "│  {} {:<60} │",
                "/help, /h".bright_yellow().bold(),
                "Show this interactive command cheat sheet reference"
            );
            println!(
                "│  {} {:<60} │",
                "/clear".bright_yellow().bold(),
                "Reset conversation history & start a fresh session"
            );
            println!(
                "│  {} {:<60} │",
                "/compact".bright_yellow().bold(),
                "Prune older turn context to conserve token budget"
            );
            println!(
                "│  {} {:<60} │",
                "/model <name>".bright_yellow().bold(),
                "Switch active LLM model on the fly"
            );
            println!(
                "│  {} {:<60} │",
                "/plan <task>".bright_yellow().bold(),
                "Generate and preview a bounded 3-chunk execution plan"
            );
            println!(
                "│  {} {:<60} │",
                "/doctor".bright_yellow().bold(),
                "Run security audit, secret scan & toolchain checks"
            );
            println!(
                "│  {} {:<60} │",
                "/usage".bright_yellow().bold(),
                "View SQLite token usage ledger, session cost & savings"
            );
            println!(
                "│  {} {:<60} │",
                "/memory".bright_yellow().bold(),
                "Inspect active project conventions from MEMORY.md"
            );
            println!(
                "│  {} {:<60} │",
                "/diff".bright_yellow().bold(),
                "Show uncommitted Git diff in the current workspace"
            );
            println!(
                "│  {} {:<60} │",
                "/exit, /quit".bright_yellow().bold(),
                "Exit interactive session safely"
            );
            println!(
                "{}",
                "╰─────────────────────────────────────────────────────────────────────────────╯"
                    .bright_cyan()
                    .bold()
            );
            println!();
        }
        "/clear" => {
            session.clear();
            println!(
                "{} {}",
                "✔".bright_green().bold(),
                "Conversation history cleared. Ready for fresh task!"
                    .bright_white()
                    .bold()
            );
        }
        "/compact" => {
            session.compact();
            println!(
                "{} {}",
                "✔".bright_green().bold(),
                "Session history compacted to latest turns to optimize token budget."
                    .bright_white()
                    .bold()
            );
        }
        "/model" => {
            if parts.len() > 1 {
                let new_model = parts[1].to_string();
                cfg.model = new_model.clone();
                session.config.model = new_model.clone();
                println!(
                    "{} Switched model to: {}",
                    "✔".bright_green().bold(),
                    new_model.bright_yellow().bold()
                );
            } else {
                println!("Current model: {}", cfg.model.bright_yellow().bold());
                println!("Usage: /model <model_name>");
            }
        }
        "/plan" => {
            if parts.len() > 1 {
                let task = parts[1..].join(" ");
                match create_task_plan(&task, &[]) {
                    Ok(plan) => {
                        println!(
                            "╭── 📋 Execution Plan ID: {} ───────────────────────────────────╮",
                            plan.id.bright_cyan().bold()
                        );
                        for chunk in &plan.chunks {
                            println!(
                                "│  {}. {} — {}",
                                chunk.sequence.to_string().bright_yellow().bold(),
                                chunk.title.bright_white().bold(),
                                chunk.goal.dimmed()
                            );
                        }
                        println!(
                            "╰─────────────────────────────────────────────────────────────────────────────╯"
                        );
                    }
                    Err(e) => eprintln!("{} {e}", "✖ Plan Error:".red().bold()),
                }
            } else {
                println!("Usage: /plan <task description>");
            }
        }
        "/doctor" => {
            println!(
                "{}",
                "╭── 🩺 Project Security & Health Audit ──────────────────────────────────────╮"
                    .bright_cyan()
                    .bold()
            );
            let report = security::scan_project(&cfg.project_root);
            for f in &report.findings {
                println!(
                    "│  [{}] [{}] {}",
                    f.severity.label(),
                    f.category.bold(),
                    f.message
                );
            }
            let (pass, info, warn, crit) = report.counts();
            println!(
                "├─────────────────────────────────────────────────────────────────────────────┤"
            );
            println!(
                "│  Summary: {} pass | {} info | {} warnings | {} critical                 │",
                pass.to_string().bright_green().bold(),
                info.to_string().bright_blue().bold(),
                warn.to_string().bright_yellow().bold(),
                crit.to_string().bright_red().bold()
            );
            println!(
                "{}",
                "╰─────────────────────────────────────────────────────────────────────────────╯"
                    .bright_cyan()
                    .bold()
            );
        }
        "/usage" => {
            println!(
                "{}",
                "╭── 📊 Local Token Usage & Savings Ledger ───────────────────────────────────╮"
                    .bright_cyan()
                    .bold()
            );
            if let Ok(ledger) = UsageLedger::init(&cfg.project_root)
                && let Ok((p, c, cost)) = ledger.summary()
            {
                println!(
                    "│  Prompt Tokens:     {:<48} │",
                    p.to_string().bright_yellow()
                );
                println!(
                    "│  Completion Tokens: {:<48} │",
                    c.to_string().bright_yellow()
                );
                println!(
                    "│  Total Lifetime:    {:<48} │",
                    (p + c).to_string().bright_green().bold()
                );
                println!("│  Estimated Cost:    ${:<47.4} │", cost);
                println!(
                    "├─────────────────────────────────────────────────────────────────────────────┤"
                );
            }
            println!(
                "│  Session Input:     {:<48} │",
                format!("{} tokens", session.total_input_tokens).bright_cyan()
            );
            println!(
                "│  Session Output:    {:<48} │",
                format!("{} tokens", session.total_output_tokens).bright_cyan()
            );
            println!(
                "{}",
                "╰─────────────────────────────────────────────────────────────────────────────╯"
                    .bright_cyan()
                    .bold()
            );
        }
        "/memory" => {
            let mem = MemoryManager::new(&cfg.project_root);
            if let Ok(content) = mem.read() {
                println!("{}", "╭── 🧠 Project Memory (MEMORY.md) ───────────────────────────────────────────╮".bright_cyan().bold());
                for line in content.lines() {
                    println!("│ {line}");
                }
                println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
            }
        }
        "/diff" => {
            let output = std::process::Command::new("git")
                .args(["diff", "--stat"])
                .current_dir(&cfg.project_root)
                .output();
            match output {
                Ok(out) => {
                    let diff_str = String::from_utf8_lossy(&out.stdout);
                    println!("{}", "╭── 🔍 Uncommitted Git Changes ─────────────────────────────────────────────╮".bright_cyan().bold());
                    if diff_str.trim().is_empty() {
                        println!("│  No uncommitted git changes detected.");
                    } else {
                        for line in diff_str.lines() {
                            println!("│  {line}");
                        }
                    }
                    println!("{}", "╰─────────────────────────────────────────────────────────────────────────────╯".bright_cyan().bold());
                }
                Err(e) => eprintln!("Git error: {e}"),
            }
        }
        "/exit" | "/quit" | "/q" => {
            println!(
                "{}",
                "👋 Exiting Deepak Bagada IND AI. Happy coding!"
                    .bright_cyan()
                    .bold()
            );
            return Ok(true);
        }
        unknown => {
            println!(
                "{} Unknown command `{unknown}`. Type `/help` for list of slash commands.",
                "⚠".bright_yellow().bold()
            );
        }
    }
    Ok(false)
}

fn print_welcome_banner(cfg: &IndConfig) {
    let border_color = |s: &str| s.bright_cyan().bold();
    let model_display = if cfg.model.is_empty() {
        "default".to_string()
    } else {
        cfg.model.clone()
    };

    println!(
        "{}",
        border_color(
            "╭─────────────────────────────────────────────────────────────────────────────╮"
        )
    );
    println!(
        "│  {}  │",
        "____  _____ _____ ____   _    _      ____    _    ____    _    ____   _  "
            .bright_cyan()
            .bold()
    );
    println!(
        "│ {} │",
        "|  _ \\| ____| ____|  _ \\ / \\  | |/ /  | __ )  / \\  / ___|  / \\  |  _ \\ / \\ "
            .bright_cyan()
            .bold()
    );
    println!(
        "│ {} │",
        "| | | |  _| |  _| | |_) / _ \\ | ' /   |  _ \\ / _ \\| |  _  / _ \\ | | | / _ \\"
            .bright_cyan()
            .bold()
    );
    println!(
        "│ {} │",
        "| |_| | |___| |___|  __/ ___ \\| . \\   | |_) / ___ \\ |_| |/ ___ \\| |_| / ___ \\"
            .bright_cyan()
            .bold()
    );
    println!(
        "│ {} │",
        "|____/|_____|_____|_| /_/   \\_\\_|\\_\\  |____/_/   \\_\\____/_/   \\_\\____/_/   \\_\\"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        border_color(
            "│                                                                             │"
        )
    );
    println!(
        "│                   {}                   │",
        "⚡ IND AI — NATIVE RUST CODING TERMINAL ⚡"
            .bright_magenta()
            .bold()
    );
    println!(
        "│                       {}                      │",
        "Created by Deepak Bagada".bright_yellow().bold()
    );
    println!(
        "{}",
        border_color(
            "├─────────────────────────────────────────────────────────────────────────────┤"
        )
    );
    println!(
        "│  {} {:<57} │",
        "📁 Project: ".bright_green().bold(),
        cfg.project_root.display().to_string().cyan()
    );
    println!(
        "│  {} {:<57} │",
        "🤖 Provider:".bright_green().bold(),
        format!(
            "{} ({})",
            cfg.provider.yellow().bold(),
            model_display.bold()
        )
    );
    println!(
        "│  {} {:<57} │",
        "⚡ Mode:    ".bright_green().bold(),
        "Tiered Model Routing + Token-Budget Context (~50% savings)"
            .to_string()
            .bright_white()
    );
    println!(
        "│  {} {:<57} │",
        "💡 Quicktip:".bright_green().bold(),
        format!(
            "Type coding prompt or {} for slash commands",
            "/help".bright_cyan().bold()
        )
    );
    println!(
        "{}",
        border_color(
            "╰─────────────────────────────────────────────────────────────────────────────╯"
        )
    );
    println!();
}
