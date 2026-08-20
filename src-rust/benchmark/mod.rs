use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use colored::*;
use serde::{Deserialize, Serialize};

use crate::context::selector::{estimate_tokens, inspect_and_select};

// ---------------------------------------------------------------------------
// Benchmark fixture definition — matches fixtures/benchmark/*.json.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkFixture {
    pub id: String,
    pub task: String,
    pub root: String,
    pub expected_files: Vec<String>,
    pub budget_tokens: usize,
}

// ---------------------------------------------------------------------------
// Benchmark result for a single fixture run.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResult {
    pub fixture_id: String,
    pub task: String,
    pub baseline_tokens: usize,
    pub selected_tokens: usize,
    pub selected_files: usize,
    pub total_files: usize,
    pub expected_files_hit: usize,
    pub expected_files_total: usize,
    pub savings_percent: f64,
    pub duration_ms: u128,
    pub pass: bool,
}

// ---------------------------------------------------------------------------
// Leaderboard entry — aggregated across all fixture runs.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub timestamp: String,
    pub fixtures_run: usize,
    pub fixtures_passed: usize,
    pub median_savings_percent: f64,
    pub mean_savings_percent: f64,
    pub total_baseline_tokens: usize,
    pub total_selected_tokens: usize,
    pub total_duration_ms: u128,
    pub results: Vec<BenchmarkResult>,
    pub verdict: String,
}

// ---------------------------------------------------------------------------
// Constants — PRD guardrails.
// ---------------------------------------------------------------------------

/// Minimum median savings required (PRD: at least 20%).
const MIN_MEDIAN_SAVINGS: f64 = 20.0;
/// Maximum regression in expected-file hit rate (PRD: no more than 10%).
const MAX_MISS_RATE: f64 = 10.0;

// ---------------------------------------------------------------------------
// Public API.
// ---------------------------------------------------------------------------

/// Discover and load all benchmark fixtures from `fixtures/benchmark/`.
pub fn load_fixtures(project_root: &Path) -> Result<Vec<BenchmarkFixture>, String> {
    let fixture_dir = project_root.join("fixtures").join("benchmark");
    if !fixture_dir.exists() {
        return Err(format!(
            "Benchmark fixtures directory not found: {}",
            fixture_dir.display()
        ));
    }

    let mut fixtures = Vec::new();
    let entries =
        fs::read_dir(&fixture_dir).map_err(|e| format!("Cannot read fixture directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let text = fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
            let fixture: BenchmarkFixture = serde_json::from_str(&text)
                .map_err(|e| format!("Invalid fixture {}: {e}", path.display()))?;
            fixtures.push(fixture);
        }
    }

    if fixtures.is_empty() {
        return Err("No benchmark fixtures found (*.json files).".to_string());
    }

    fixtures.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(fixtures)
}

/// Run a single benchmark fixture. Returns the result.
pub fn run_fixture(
    project_root: &Path,
    fixture: &BenchmarkFixture,
) -> Result<BenchmarkResult, String> {
    let fixture_root = project_root
        .join("fixtures")
        .join("benchmark")
        .join(&fixture.root);

    if !fixture_root.exists() {
        return Err(format!(
            "Fixture root not found: {}",
            fixture_root.display()
        ));
    }

    let start = Instant::now();

    // Calculate baseline: total tokens across ALL files in the fixture repo.
    let baseline_tokens = count_all_tokens(&fixture_root)?;

    // Run IND context selection with the fixture's task and budget.
    let selection = inspect_and_select(&fixture_root, &fixture.task, fixture.budget_tokens)?;

    let duration_ms = start.elapsed().as_millis();

    // Count how many expected files were included.
    let selected_paths: Vec<&str> = selection
        .selected
        .iter()
        .map(|f| f.relative_path.as_str())
        .collect();

    let expected_files_hit = fixture
        .expected_files
        .iter()
        .filter(|ef| {
            let normalized = ef.replace('\\', "/");
            selected_paths.iter().any(|sp| {
                let sp_normalized = sp.replace('\\', "/");
                sp_normalized == normalized || sp_normalized.ends_with(&format!("/{normalized}"))
            })
        })
        .count();

    let savings_percent = if baseline_tokens > 0 {
        (1.0 - (selection.estimated_tokens as f64 / baseline_tokens as f64)) * 100.0
    } else {
        0.0
    };

    // Pass if savings > 0 and expected file hit rate >= (100 - MAX_MISS_RATE)%.
    let hit_rate = if fixture.expected_files.is_empty() {
        100.0
    } else {
        (expected_files_hit as f64 / fixture.expected_files.len() as f64) * 100.0
    };
    let pass = savings_percent >= 0.0 && hit_rate >= (100.0 - MAX_MISS_RATE);

    Ok(BenchmarkResult {
        fixture_id: fixture.id.clone(),
        task: fixture.task.clone(),
        baseline_tokens,
        selected_tokens: selection.estimated_tokens,
        selected_files: selection.selected.len(),
        total_files: selection.selected.len() + selection.omitted.len(),
        expected_files_hit,
        expected_files_total: fixture.expected_files.len(),
        savings_percent,
        duration_ms,
        pass,
    })
}

/// Run all benchmark fixtures and return the leaderboard entry.
pub fn run_all(project_root: &Path) -> Result<LeaderboardEntry, String> {
    let fixtures = load_fixtures(project_root)?;
    let mut results = Vec::new();

    for fixture in &fixtures {
        results.push(run_fixture(project_root, fixture)?);
    }

    let fixtures_passed = results.iter().filter(|r| r.pass).count();
    let total_baseline: usize = results.iter().map(|r| r.baseline_tokens).sum();
    let total_selected: usize = results.iter().map(|r| r.selected_tokens).sum();
    let total_duration: u128 = results.iter().map(|r| r.duration_ms).sum();

    let mut savings_list: Vec<f64> = results.iter().map(|r| r.savings_percent).collect();
    savings_list.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let median_savings = if savings_list.is_empty() {
        0.0
    } else {
        let mid = savings_list.len() / 2;
        if savings_list.len().is_multiple_of(2) {
            (savings_list[mid - 1] + savings_list[mid]) / 2.0
        } else {
            savings_list[mid]
        }
    };

    let mean_savings = if savings_list.is_empty() {
        0.0
    } else {
        savings_list.iter().sum::<f64>() / savings_list.len() as f64
    };

    let verdict = if median_savings >= MIN_MEDIAN_SAVINGS && fixtures_passed == results.len() {
        "PASS — meets PRD guardrail (≥20% median savings, ≤10% miss rate)".to_string()
    } else if median_savings >= MIN_MEDIAN_SAVINGS {
        format!(
            "PARTIAL — median savings OK ({median_savings:.1}%) but {}/{} fixtures passed",
            fixtures_passed,
            results.len()
        )
    } else {
        format!("BELOW TARGET — median savings {median_savings:.1}% < {MIN_MEDIAN_SAVINGS}% target")
    };

    Ok(LeaderboardEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        fixtures_run: results.len(),
        fixtures_passed,
        median_savings_percent: median_savings,
        mean_savings_percent: mean_savings,
        total_baseline_tokens: total_baseline,
        total_selected_tokens: total_selected,
        total_duration_ms: total_duration,
        results,
        verdict,
    })
}

/// Format and print a benchmark result to the terminal.
pub fn print_result(result: &BenchmarkResult) {
    let status = if result.pass {
        "PASS".green().bold()
    } else {
        "FAIL".red().bold()
    };

    println!(
        "  [{}] {} — savings {:.1}%, {}/{} files selected, {}/{} expected files hit ({}ms)",
        status,
        result.fixture_id,
        result.savings_percent,
        result.selected_files,
        result.total_files,
        result.expected_files_hit,
        result.expected_files_total,
        result.duration_ms,
    );
}

/// Generate a leaderboard markdown report and write it to `output/benchmark/`.
pub fn write_leaderboard_report(
    project_root: &Path,
    entry: &LeaderboardEntry,
) -> Result<PathBuf, String> {
    let output_dir = project_root.join("output").join("benchmark");
    fs::create_dir_all(&output_dir).map_err(|e| format!("Cannot create output directory: {e}"))?;

    let report_path = output_dir.join("leaderboard.md");

    let mut report = String::new();
    report.push_str("# IND Context Benchmark Leaderboard\n\n");
    report.push_str(&format!("> Generated: {}\n\n", entry.timestamp));
    report.push_str("## Summary\n\n");
    report.push_str("| Metric | Value |\n|--------|-------|\n");
    report.push_str(&format!("| Fixtures run | {} |\n", entry.fixtures_run));
    report.push_str(&format!(
        "| Fixtures passed | {} |\n",
        entry.fixtures_passed
    ));
    report.push_str(&format!(
        "| Median savings | {:.1}% |\n",
        entry.median_savings_percent
    ));
    report.push_str(&format!(
        "| Mean savings | {:.1}% |\n",
        entry.mean_savings_percent
    ));
    report.push_str(&format!(
        "| Total baseline tokens | {} |\n",
        entry.total_baseline_tokens
    ));
    report.push_str(&format!(
        "| Total selected tokens | {} |\n",
        entry.total_selected_tokens
    ));
    report.push_str(&format!(
        "| Total duration | {}ms |\n",
        entry.total_duration_ms
    ));
    report.push_str(&format!("| **Verdict** | {} |\n", entry.verdict));

    report.push_str("\n## Results\n\n");
    report
        .push_str("| Fixture | Baseline | Selected | Savings | Expected Hit | Pass | Duration |\n");
    report
        .push_str("|---------|----------|----------|---------|--------------|------|----------|\n");

    for r in &entry.results {
        report.push_str(&format!(
            "| {} | {} | {} | {:.1}% | {}/{} | {} | {}ms |\n",
            r.fixture_id,
            r.baseline_tokens,
            r.selected_tokens,
            r.savings_percent,
            r.expected_files_hit,
            r.expected_files_total,
            if r.pass { "✅" } else { "❌" },
            r.duration_ms,
        ));
    }

    report.push_str("\n---\n\n");
    report.push_str(&format!(
        "> PRD Guardrail: ≥{MIN_MEDIAN_SAVINGS}% median input-token reduction, ≤{MAX_MISS_RATE}% expected-file miss rate.\n"
    ));

    fs::write(&report_path, &report)
        .map_err(|e| format!("Cannot write leaderboard report: {e}"))?;

    // Also write the raw JSONL entry for machine consumption.
    let jsonl_path = output_dir.join("leaderboard.jsonl");
    let json_line =
        serde_json::to_string(entry).map_err(|e| format!("Cannot serialize entry: {e}"))?;

    let mut existing = fs::read_to_string(&jsonl_path).unwrap_or_default();
    existing.push_str(&json_line);
    existing.push('\n');
    fs::write(&jsonl_path, &existing)
        .map_err(|e| format!("Cannot write leaderboard JSONL: {e}"))?;

    Ok(report_path)
}

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

/// Count total tokens across all text files in a directory tree.
fn count_all_tokens(root: &Path) -> Result<usize, String> {
    let mut total = 0;
    count_tokens_recursive(root, &mut total)?;
    Ok(total)
}

fn count_tokens_recursive(dir: &Path, total: &mut usize) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("Cannot read {}: {e}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden dirs and common non-source dirs.
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }

        if path.is_dir() {
            count_tokens_recursive(&path, total)?;
            continue;
        }

        // Only count text files.
        if let Ok(content) = fs::read_to_string(&path) {
            *total += estimate_tokens(&content);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_fixtures_from_project() {
        // This test relies on the actual fixtures directory.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixtures = load_fixtures(root);
        match fixtures {
            Ok(f) => {
                assert!(!f.is_empty(), "Expected at least one fixture");
                assert!(f.iter().any(|fix| fix.id == "auth-task"));
                assert!(f.iter().any(|fix| fix.id == "ui-task"));
            }
            Err(e) => panic!("Failed to load fixtures: {e}"),
        }
    }

    #[test]
    fn runs_auth_fixture() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixtures = load_fixtures(root).unwrap();
        let auth = fixtures.iter().find(|f| f.id == "auth-task").unwrap();
        let result = run_fixture(root, auth).unwrap();
        assert_eq!(result.fixture_id, "auth-task");
        assert!(result.baseline_tokens > 0, "Baseline tokens should be > 0");
        assert!(result.duration_ms < 5000, "Benchmark should be fast");
    }

    #[test]
    fn runs_all_fixtures() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let entry = run_all(root).unwrap();
        assert!(entry.fixtures_run >= 2, "Expected at least 2 fixtures");
        assert!(!entry.verdict.is_empty(), "Verdict should not be empty");
    }

    #[test]
    fn count_tokens_works() {
        let dir = std::env::temp_dir().join(format!("ind-bench-tokens-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.rs"), "fn main() {}").unwrap();
        fs::write(dir.join("b.rs"), "fn other() {}").unwrap();
        let total = count_all_tokens(&dir).unwrap();
        assert!(total > 0, "Should count some tokens");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn savings_percent_is_reasonable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let entry = run_all(root).unwrap();
        // Savings should be between -100% and 100%.
        for r in &entry.results {
            assert!(
                r.savings_percent > -100.0 && r.savings_percent <= 100.0,
                "Savings {:.1}% out of range for {}",
                r.savings_percent,
                r.fixture_id
            );
        }
    }
}
