use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "affected",
    version,
    about = "Detect affected packages. Run only what matters. Language-agnostic monorepo tool."
)]
struct Cli {
    /// Path to the project root (default: current directory)
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,

    /// Increase verbosity (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Path to a custom config file (default: .affected.toml)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run tests for affected packages
    Test {
        /// Base git ref to compare against (branch, tag, or commit)
        #[arg(long, required_unless_present = "merge_base")]
        base: Option<String>,

        /// Use merge-base between HEAD and this branch (more accurate for PRs)
        #[arg(long)]
        merge_base: Option<String>,

        /// Show what would be tested without executing
        #[arg(long)]
        dry_run: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Only test packages matching this glob pattern
        #[arg(long)]
        filter: Option<String>,

        /// Skip packages matching this glob pattern
        #[arg(long)]
        skip: Option<String>,

        /// Show why each package is affected
        #[arg(long)]
        explain: bool,

        /// Number of parallel test jobs (0 = auto, 1 = sequential)
        #[arg(long, short = 'j', default_value = "1")]
        jobs: usize,

        /// Timeout per package in seconds
        #[arg(long)]
        timeout: Option<u64>,

        /// Write JUnit XML results to this file
        #[arg(long)]
        junit: Option<PathBuf>,
    },

    /// List affected packages without running tests
    List {
        /// Base git ref to compare against
        #[arg(long, required_unless_present = "merge_base")]
        base: Option<String>,

        /// Use merge-base between HEAD and this branch
        #[arg(long)]
        merge_base: Option<String>,

        /// Output as JSON (for CI integration)
        #[arg(long)]
        json: bool,

        /// Only include packages matching this glob pattern
        #[arg(long)]
        filter: Option<String>,

        /// Exclude packages matching this glob pattern
        #[arg(long)]
        skip: Option<String>,

        /// Show why each package is affected
        #[arg(long)]
        explain: bool,
    },

    /// Display the project dependency graph
    Graph {
        /// Output in DOT format (for Graphviz)
        #[arg(long)]
        dot: bool,
    },

    /// Show detected project type and packages
    Detect,

    /// Output affected packages for CI systems (GitHub Actions)
    Ci {
        /// Base git ref to compare against
        #[arg(long, required_unless_present = "merge_base")]
        base: Option<String>,

        /// Use merge-base between HEAD and this branch
        #[arg(long)]
        merge_base: Option<String>,

        /// Only include packages matching this glob pattern
        #[arg(long)]
        filter: Option<String>,

        /// Exclude packages matching this glob pattern
        #[arg(long)]
        skip: Option<String>,
    },

    /// Run a custom command for each affected package
    Run {
        /// Command template to execute (use {package} as placeholder)
        command: String,

        /// Base git ref to compare against (branch, tag, or commit)
        #[arg(long, required_unless_present = "merge_base")]
        base: Option<String>,

        /// Use merge-base between HEAD and this branch (more accurate for PRs)
        #[arg(long)]
        merge_base: Option<String>,

        /// Show what would be run without executing
        #[arg(long)]
        dry_run: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Only include packages matching this glob pattern
        #[arg(long)]
        filter: Option<String>,

        /// Exclude packages matching this glob pattern
        #[arg(long)]
        skip: Option<String>,

        /// Show why each package is affected
        #[arg(long)]
        explain: bool,

        /// Number of parallel jobs (0 = auto, 1 = sequential)
        #[arg(long, short = 'j', default_value = "1")]
        jobs: usize,

        /// Timeout per package in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --no-color
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    // Initialize tracing
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .with_target(false)
        .without_time()
        .init();

    let root = std::fs::canonicalize(&cli.root)?;

    match cli.command {
        Commands::Test {
            base,
            merge_base,
            dry_run,
            json,
            filter,
            skip,
            explain,
            jobs,
            timeout,
            junit,
        } => {
            let base_ref = resolve_base(&root, base, merge_base)?;
            cmd_test(
                &root,
                &base_ref,
                dry_run,
                json,
                filter.as_deref(),
                skip.as_deref(),
                explain,
                jobs,
                timeout,
                junit,
                cli.config.as_deref(),
                cli.quiet,
            )
        }
        Commands::List {
            base,
            merge_base,
            json,
            filter,
            skip,
            explain,
        } => {
            let base_ref = resolve_base(&root, base, merge_base)?;
            cmd_list(
                &root,
                &base_ref,
                json,
                filter.as_deref(),
                skip.as_deref(),
                explain,
                cli.quiet,
            )
        }
        Commands::Graph { dot } => cmd_graph(&root, dot),
        Commands::Detect => cmd_detect(&root),
        Commands::Ci {
            base,
            merge_base,
            filter,
            skip,
        } => {
            let base_ref = resolve_base(&root, base, merge_base)?;
            cmd_ci(&root, &base_ref, filter.as_deref(), skip.as_deref())
        }
        Commands::Run {
            command,
            base,
            merge_base,
            dry_run,
            json,
            filter,
            skip,
            explain,
            jobs,
            timeout,
        } => {
            let base_ref = resolve_base(&root, base, merge_base)?;
            cmd_run(
                &root,
                &base_ref,
                &command,
                dry_run,
                json,
                filter.as_deref(),
                skip.as_deref(),
                explain,
                jobs,
                timeout,
                cli.quiet,
            )
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "affected", &mut std::io::stdout());
            Ok(())
        }
    }
}

/// Resolve the base ref from --base or --merge-base flags.
fn resolve_base(
    root: &std::path::Path,
    base: Option<String>,
    merge_base: Option<String>,
) -> Result<String> {
    if let Some(mb) = merge_base {
        affected_core::find_merge_base(root, &mb)
    } else {
        Ok(base.unwrap_or_else(|| "HEAD".to_string()))
    }
}

fn load_config(
    root: &std::path::Path,
    config_path: Option<&std::path::Path>,
) -> Result<affected_core::config::Config> {
    match config_path {
        Some(path) => affected_core::config::Config::load_from(path),
        None => affected_core::config::Config::load(root),
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_test(
    root: &std::path::Path,
    base: &str,
    dry_run: bool,
    json: bool,
    filter: Option<&str>,
    skip: Option<&str>,
    explain: bool,
    jobs: usize,
    timeout: Option<u64>,
    junit: Option<PathBuf>,
    config_path: Option<&std::path::Path>,
    quiet: bool,
) -> Result<()> {
    let config = load_config(root, config_path)?;
    let result = affected_core::find_affected_with_options(root, base, explain, filter, skip)?;

    if result.affected.is_empty() {
        if !json && !quiet {
            println!("{}", "No packages affected.".dimmed());
        }
        if json {
            let empty = affected_core::types::TestOutputJson {
                affected: vec![],
                results: vec![],
                summary: affected_core::types::TestSummaryJson {
                    passed: 0,
                    failed: 0,
                    total: 0,
                    duration_ms: 0,
                },
            };
            println!("{}", serde_json::to_string_pretty(&empty)?);
        }
        return Ok(());
    }

    if !json && !quiet {
        println!(
            "{} {} affected package(s) (out of {} total, {} files changed):",
            "Testing".bold().cyan(),
            result.affected.len(),
            result.total_packages,
            result.changed_files,
        );
        if explain {
            print_explanations(&result);
        }
        println!();
    }

    let resolver = affected_core::resolvers::detect_resolver(root)?;
    let ecosystem = resolver.ecosystem();

    let commands: Vec<_> = result
        .affected
        .iter()
        .filter(|name| {
            // Skip packages marked skip=true in config
            !config
                .package_config(name)
                .and_then(|pc| pc.skip)
                .unwrap_or(false)
        })
        .map(|name| {
            let pkg_id = affected_core::types::PackageId(name.clone());
            // Priority: per-package config > ecosystem config > resolver default
            let cmd = config
                .package_config(name)
                .and_then(|pc| pc.test.as_ref())
                .map(|t| {
                    t.replace("{package}", name)
                        .split_whitespace()
                        .map(String::from)
                        .collect()
                })
                .or_else(|| config.test_command_for(ecosystem, name))
                .unwrap_or_else(|| resolver.test_command(&pkg_id));
            (pkg_id, cmd)
        })
        .collect();

    let timeout_dur = timeout.map(std::time::Duration::from_secs);
    let runner = affected_core::runner::Runner::new(affected_core::runner::RunnerConfig {
        root: root.to_path_buf(),
        dry_run,
        timeout: timeout_dur,
        jobs,
        json,
        quiet,
    });

    let results = runner.run_tests(commands)?;

    if json {
        let json_output = affected_core::runner::results_to_json(&result.affected, &results);
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        affected_core::runner::print_summary(&results);
    }

    // Write JUnit XML if requested
    if let Some(junit_path) = junit {
        let junit_xml = affected_core::runner::results_to_junit(&results);
        std::fs::write(&junit_path, junit_xml)?;
        if !quiet {
            println!("  JUnit results written to {}", junit_path.display());
        }
    }

    let any_failed = results.iter().any(|r| !r.success);
    if any_failed {
        std::process::exit(1);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_run(
    root: &std::path::Path,
    base: &str,
    command_template: &str,
    dry_run: bool,
    json: bool,
    filter: Option<&str>,
    skip: Option<&str>,
    explain: bool,
    jobs: usize,
    timeout: Option<u64>,
    quiet: bool,
) -> Result<()> {
    let result = affected_core::find_affected_with_options(root, base, explain, filter, skip)?;

    if result.affected.is_empty() {
        if !json && !quiet {
            println!("{}", "No packages affected.".dimmed());
        }
        if json {
            let empty = affected_core::types::TestOutputJson {
                affected: vec![],
                results: vec![],
                summary: affected_core::types::TestSummaryJson {
                    passed: 0,
                    failed: 0,
                    total: 0,
                    duration_ms: 0,
                },
            };
            println!("{}", serde_json::to_string_pretty(&empty)?);
        }
        return Ok(());
    }

    if !json && !quiet {
        println!(
            "{} command for {} affected package(s) (out of {} total, {} files changed):",
            "Running".bold().cyan(),
            result.affected.len(),
            result.total_packages,
            result.changed_files,
        );
        if explain {
            print_explanations(&result);
        }
        println!();
    }

    let commands: Vec<_> = result
        .affected
        .iter()
        .map(|name| {
            let pkg_id = affected_core::types::PackageId(name.clone());
            let cmd: Vec<String> = command_template
                .replace("{package}", name)
                .split_whitespace()
                .map(String::from)
                .collect();
            (pkg_id, cmd)
        })
        .collect();

    let timeout_dur = timeout.map(std::time::Duration::from_secs);
    let runner = affected_core::runner::Runner::new(affected_core::runner::RunnerConfig {
        root: root.to_path_buf(),
        dry_run,
        timeout: timeout_dur,
        jobs,
        json,
        quiet,
    });

    let results = runner.run_tests(commands)?;

    if json {
        let json_output = affected_core::runner::results_to_json(&result.affected, &results);
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        affected_core::runner::print_summary(&results);
    }

    let any_failed = results.iter().any(|r| !r.success);
    if any_failed {
        std::process::exit(1);
    }

    Ok(())
}

fn cmd_list(
    root: &std::path::Path,
    base: &str,
    json: bool,
    filter: Option<&str>,
    skip: Option<&str>,
    explain: bool,
    quiet: bool,
) -> Result<()> {
    let result = affected_core::find_affected_with_options(root, base, explain, filter, skip)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if result.affected.is_empty() {
        if !quiet {
            println!("{}", "No packages affected.".dimmed());
        }
        return Ok(());
    }

    if !quiet {
        println!(
            "{} affected package(s) (base: {}, {} files changed):\n",
            result.affected.len().to_string().bold(),
            base.cyan(),
            result.changed_files,
        );
    }

    if explain {
        print_explanations(&result);
    } else {
        for name in &result.affected {
            println!("  {} {}", "●".green(), name);
        }
    }

    Ok(())
}

fn cmd_graph(root: &std::path::Path, dot: bool) -> Result<()> {
    let (_resolver, project_graph) = affected_core::resolve_project(root)?;
    let dep_graph = affected_core::graph::DepGraph::from_project_graph(&project_graph);

    // Check for cycles
    if dep_graph.has_cycles() {
        let cycles = dep_graph.find_cycles();
        eprintln!(
            "{} Dependency cycles detected ({}):",
            "Warning:".yellow().bold(),
            cycles.len()
        );
        for cycle in &cycles {
            let names: Vec<_> = cycle.iter().map(|p| p.0.as_str()).collect();
            eprintln!("  {} {}", "⟳".yellow(), names.join(" → "));
        }
        eprintln!();
    }

    if dot {
        println!("{}", dep_graph.to_dot());
        return Ok(());
    }

    let edges = dep_graph.edges();
    if edges.is_empty() {
        println!("{}", "No dependencies between packages.".dimmed());
        return Ok(());
    }

    println!("{}\n", "Dependency Graph:".bold());
    for (from, to) in &edges {
        println!("  {} {} {}", from.to_string().cyan(), "→".dimmed(), to);
    }

    Ok(())
}

fn cmd_detect(root: &std::path::Path) -> Result<()> {
    let ecosystems = affected_core::detect::detect_ecosystems(root)?;
    let (resolver, project_graph) = affected_core::resolve_project(root)?;

    println!("{} {}\n", "Ecosystem:".bold(), resolver.ecosystem());
    println!(
        "Detected: {}",
        ecosystems
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    println!(
        "{} ({} found):\n",
        "Packages".bold(),
        project_graph.packages.len()
    );

    let mut names: Vec<_> = project_graph
        .packages
        .values()
        .map(|p| (&p.name, &p.path))
        .collect();
    names.sort_by_key(|(n, _)| (*n).clone());

    for (name, path) in names {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        println!("  {} {} {}", "●".green(), name.cyan(), rel.dimmed());
    }

    Ok(())
}

fn cmd_ci(
    root: &std::path::Path,
    base: &str,
    filter: Option<&str>,
    skip: Option<&str>,
) -> Result<()> {
    let result = affected_core::find_affected_with_options(root, base, false, filter, skip)?;

    let affected_csv = result.affected.join(",");
    let count = result.affected.len();
    let has_affected = !result.affected.is_empty();

    // Build matrix JSON for GitHub Actions dynamic matrix strategy
    let matrix = serde_json::json!({"package": &result.affected});
    let matrix_str = serde_json::to_string(&matrix)?;

    // Output for GitHub Actions
    if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
        let mut content = String::new();
        content.push_str(&format!("affected={}\n", affected_csv));
        content.push_str(&format!("count={}\n", count));
        content.push_str(&format!("has_affected={}\n", has_affected));
        content.push_str(&format!(
            "affected_json={}\n",
            serde_json::to_string(&result.affected)?
        ));
        content.push_str(&format!("matrix={}\n", matrix_str));
        std::fs::write(output_file, content)?;
    }

    // Also print to stdout for other CI systems
    println!("affected={}", affected_csv);
    println!("count={}", count);
    println!("has_affected={}", has_affected);
    println!("matrix={}", matrix_str);

    Ok(())
}

fn print_explanations(result: &affected_core::types::AffectedResult) {
    if let Some(explanations) = &result.explanations {
        for entry in explanations {
            match &entry.reason {
                affected_core::types::ExplainReason::DirectlyChanged { files } => {
                    println!(
                        "  {} {} {}",
                        "●".yellow(),
                        entry.package.cyan(),
                        format!("(directly changed: {})", files.join(", ")).dimmed()
                    );
                }
                affected_core::types::ExplainReason::TransitivelyAffected { chain } => {
                    println!(
                        "  {} {} {}",
                        "●".red(),
                        entry.package.cyan(),
                        format!("(depends on: {})", chain.join(" → ")).dimmed()
                    );
                }
            }
        }
    }
}
