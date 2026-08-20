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
        let prompt_str = format!("{} ", "ind >".bold().cyan());
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
                        print!("{} ", "ind:".bold().green());
                        let _ = io::stdout().flush();

                        let res = session
                            .run_turn(provider.as_ref(), trimmed, |token| {
                                print!("{token}");
                                let _ = io::stdout().flush();
                            })
                            .await;

                        println!("\n");
                        if let Err(e) = res {
                            eprintln!("{} {e}", "Agent Error:".red().bold());
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {e}", "Provider Config Error:".red().bold());
                        eprintln!(
                            "Tip: Set {} or run {} to switch model/provider.",
                            "OPENAI_API_KEY".bold().yellow(),
                            "/model".cyan()
                        );
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C (Type /exit or Ctrl+D to quit)".yellow());
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".cyan());
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
            println!("{}", "── Available IND Slash Commands ──".bold().cyan());
            println!("  {}            Show this help reference", "/help".bold());
            println!("  {}           Clear conversation history", "/clear".bold());
            println!(
                "  {}         Prune older turns to save tokens",
                "/compact".bold()
            );
            println!("  {} <model>   Switch active LLM model", "/model".bold());
            println!(
                "  {} <task>     Generate and preview a 3-chunk task plan",
                "/plan".bold()
            );
            println!(
                "  {}           Run project diagnostics & security audit",
                "/doctor".bold()
            );
            println!(
                "  {}            View token usage and cost ledger",
                "/usage".bold()
            );
            println!(
                "  {}           View project conventions from MEMORY.md",
                "/memory".bold()
            );
            println!("  {}             Show uncommitted git diff", "/diff".bold());
            println!(
                "  {}     Exit the interactive session",
                "/exit, /quit".bold()
            );
            println!();
        }
        "/clear" => {
            session.clear();
            println!("{}", "Session history cleared.".green());
        }
        "/compact" => {
            session.compact();
            println!(
                "{}",
                "Session history compacted to latest turns to save tokens.".green()
            );
        }
        "/model" => {
            if parts.len() > 1 {
                let new_model = parts[1].to_string();
                cfg.model = new_model.clone();
                session.config.model = new_model.clone();
                println!("{} Switched model to: {}", "✔".green(), new_model.bold());
            } else {
                println!("Current model: {}", cfg.model.bold());
                println!("Usage: /model <model_name>");
            }
        }
        "/plan" => {
            if parts.len() > 1 {
                let task = parts[1..].join(" ");
                match create_task_plan(&task, &[]) {
                    Ok(plan) => {
                        println!("{} {}", "Plan ID:".cyan(), plan.id);
                        for chunk in &plan.chunks {
                            println!(
                                "  {}. {} — {}",
                                chunk.sequence,
                                chunk.title.bold(),
                                chunk.goal
                            );
                        }
                    }
                    Err(e) => eprintln!("{} {e}", "Plan Error:".red()),
                }
            } else {
                println!("Usage: /plan <task description>");
            }
        }
        "/doctor" => {
            println!("{}", "Running project security diagnostics...".cyan());
            let report = security::scan_project(&cfg.project_root);
            for f in &report.findings {
                println!("  [{}] [{}] {}", f.severity.label(), f.category, f.message);
            }
            let (pass, info, warn, crit) = report.counts();
            println!(
                "\n  Summary: {} pass, {} info, {} warnings, {} critical",
                pass.to_string().green(),
                info.to_string().blue(),
                warn.to_string().yellow(),
                crit.to_string().red()
            );
        }
        "/usage" => {
            println!("{}", "Local Usage Summary:".bold().cyan());
            if let Ok(ledger) = UsageLedger::init(&cfg.project_root)
                && let Ok((p, c, cost)) = ledger.summary()
            {
                println!("  Prompt Tokens:     {p}");
                println!("  Completion Tokens: {c}");
                println!("  Total Tokens:      {}", p + c);
                println!("  Estimated Cost:    ${cost:.4}");
            }
            println!("  Session Input:     {} tokens", session.total_input_tokens);
            println!(
                "  Session Output:    {} tokens",
                session.total_output_tokens
            );
        }
        "/memory" => {
            let mem = MemoryManager::new(&cfg.project_root);
            if let Ok(content) = mem.read() {
                println!("{}", "Project Memory (MEMORY.md):".bold().cyan());
                println!("{content}");
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
                    if diff_str.trim().is_empty() {
                        println!("No uncommitted git changes.");
                    } else {
                        println!("{diff_str}");
                    }
                }
                Err(e) => eprintln!("Git error: {e}"),
            }
        }
        "/exit" | "/quit" | "/q" => {
            println!("{}", "Exiting IND. Happy coding!".cyan());
            return Ok(true);
        }
        unknown => {
            println!(
                "{} Unknown command `{unknown}`. Type `/help` for list of commands.",
                "⚠".yellow()
            );
        }
    }
    Ok(false)
}

fn print_welcome_banner(cfg: &IndConfig) {
    println!(
        "{}",
        "==================================================".cyan()
    );
    println!(
        "{} {}",
        "IND AI Coding Agent".bold().green(),
        "(Native Rust Terminal REPL)".bold()
    );
    println!(
        "  Project:  {}",
        cfg.project_root.display().to_string().cyan()
    );
    println!("  Provider: {}", cfg.provider.yellow());
    println!(
        "  Model:    {}",
        if cfg.model.is_empty() {
            "default".to_string()
        } else {
            cfg.model.clone()
        }
        .bold()
    );
    println!(
        "Type your coding prompt directly, or {} for commands.",
        "/help".bold().cyan()
    );
    println!(
        "{}",
        "==================================================".cyan()
    );
    println!();
}
