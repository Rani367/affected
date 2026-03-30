use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
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

        /// Base git ref to highlight affected packages in the graph
        #[arg(long)]
        base: Option<String>,

        /// Use merge-base between HEAD and this branch
        #[arg(long)]
        merge_base: Option<String>,
    },

    /// Show detected project type and packages
    Detect,

    /// Output affected packages for CI systems
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

        /// CI platform output format
        #[arg(long, value_enum, default_value = "github")]
        format: CiFormat,
    },

    /// Initialize a .affected.toml config file
    Init {
        /// Skip interactive prompts and use auto-detected defaults
        #[arg(long)]
        non_interactive: bool,
    },

    /// Watch for file changes and re-run a command
    Watch {
        /// Subcommand to re-run on changes (test, list, or run)
        #[command(subcommand)]
        subcommand: WatchableCommand,

        /// Debounce interval in milliseconds
        #[arg(long, default_value = "500", global = true)]
        debounce: u64,
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

/// CI platform output format
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CiFormat {
    /// GitHub Actions ($GITHUB_OUTPUT)
    Github,
    /// GitLab CI (dotenv artifact)
    Gitlab,
    /// CircleCI ($BASH_ENV)
    Circleci,
    /// Azure Pipelines (##vso logging commands)
    Azure,
    /// Plain key=value to stdout
    Generic,
}

/// Subcommands that can be re-run in watch mode
#[derive(Subcommand)]
enum WatchableCommand {
    /// Watch and re-run tests for affected packages
    Test {
        #[arg(long, required_unless_present = "merge_base")]
        base: Option<String>,
        #[arg(long)]
        merge_base: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long)]
        skip: Option<String>,
        #[arg(long)]
        explain: bool,
        #[arg(long, short = 'j', default_value = "1")]
        jobs: usize,
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Watch and re-list affected packages
    List {
        #[arg(long, required_unless_present = "merge_base")]
        base: Option<String>,
        #[arg(long)]
        merge_base: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long)]
        skip: Option<String>,
        #[arg(long)]
        explain: bool,
    },
    /// Watch and re-run a custom command
    Run {
        command: String,
        #[arg(long, required_unless_present = "merge_base")]
        base: Option<String>,
        #[arg(long)]
        merge_base: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long)]
        skip: Option<String>,
        #[arg(long)]
        explain: bool,
        #[arg(long, short = 'j', default_value = "1")]
        jobs: usize,
        #[arg(long)]
        timeout: Option<u64>,
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

    let root = dunce::canonicalize(&cli.root)?;

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
        Commands::Graph {
            dot,
            base,
            merge_base,
        } => {
            let base_ref = match (&base, &merge_base) {
                (Some(_), _) | (_, Some(_)) => {
                    Some(resolve_base(&root, base, merge_base)?)
                }
                _ => None,
            };
            cmd_graph(&root, dot, base_ref.as_deref())
        }
        Commands::Detect => cmd_detect(&root),
        Commands::Ci {
            base,
            merge_base,
            filter,
            skip,
            format,
        } => {
            let base_ref = resolve_base(&root, base, merge_base)?;
            cmd_ci(
                &root,
                &base_ref,
                filter.as_deref(),
                skip.as_deref(),
                format,
            )
        }
        Commands::Init { non_interactive } => cmd_init(&root, non_interactive),
        Commands::Watch {
            subcommand,
            debounce,
        } => cmd_watch(&root, subcommand, debounce, cli.config.as_deref(), cli.quiet),
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

fn cmd_graph(root: &std::path::Path, dot: bool, base: Option<&str>) -> Result<()> {
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

    // Get affected packages if base is specified
    let affected_set: std::collections::HashSet<String> = if let Some(base_ref) = base {
        let result =
            affected_core::find_affected_with_options(root, base_ref, false, None, None)?;
        result.affected.into_iter().collect()
    } else {
        std::collections::HashSet::new()
    };

    // Build adjacency list: package -> list of packages it depends on
    let edges = dep_graph.edges();
    let all_packages: Vec<String> = {
        let mut names: Vec<_> = project_graph.packages.keys().map(|k| k.0.clone()).collect();
        names.sort();
        names
    };

    if all_packages.is_empty() {
        println!("{}", "No packages found.".dimmed());
        return Ok(());
    }

    // Build children map (reverse of depends-on: who depends on me)
    let mut children: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut has_parent = std::collections::HashSet::new();

    for (from, to) in &edges {
        children
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
        has_parent.insert(to.to_string());
    }

    // Sort children for consistent output
    for v in children.values_mut() {
        v.sort();
    }

    // Root nodes: packages that nothing depends on (no incoming edges)
    // Actually for a dependency tree, roots are packages that DEPEND on others but nobody depends on them
    // i.e., top-level apps/CLIs. Let's show packages that have no incoming edges.
    let mut roots: Vec<&String> = all_packages
        .iter()
        .filter(|p| !has_parent.contains(*p))
        .collect();
    roots.sort();

    let total = all_packages.len();
    if !affected_set.is_empty() {
        println!(
            "{} ({} packages, {} affected):\n",
            "Dependency Graph".bold(),
            total,
            affected_set.len(),
        );
    } else {
        println!("{} ({} packages):\n", "Dependency Graph".bold(), total);
    }

    for (i, root_pkg) in roots.iter().enumerate() {
        let is_last_root = i == roots.len() - 1;
        render_tree_node(root_pkg, "", is_last_root, &children, &affected_set, &mut std::collections::HashSet::new());
    }

    // Show isolated packages (no edges at all)
    let isolated: Vec<&String> = all_packages
        .iter()
        .filter(|p| !has_parent.contains(*p) && !children.contains_key(*p))
        .filter(|p| !roots.contains(p))
        .collect();

    if !isolated.is_empty() {
        println!();
        for name in &isolated {
            let marker = if affected_set.contains(*name) {
                format!("  {}", "●".yellow())
            } else {
                String::new()
            };
            println!("  {}{} {}", name.cyan(), marker, "(no dependencies)".dimmed());
        }
    }

    Ok(())
}

fn render_tree_node(
    name: &str,
    prefix: &str,
    is_last: bool,
    children: &std::collections::HashMap<String, Vec<String>>,
    affected: &std::collections::HashSet<String>,
    visited: &mut std::collections::HashSet<String>,
) {
    let connector = if prefix.is_empty() {
        "  "
    } else if is_last {
        "  └── "
    } else {
        "  ├── "
    };

    let marker = if affected.contains(name) {
        format!("  {}", "●".yellow())
    } else {
        String::new()
    };

    println!("{}{}{}{}", prefix, connector, name.cyan(), marker);

    if visited.contains(name) {
        return; // prevent infinite loops on cycles
    }
    visited.insert(name.to_string());

    let child_prefix = if prefix.is_empty() {
        String::new()
    } else if is_last {
        format!("{}      ", prefix)
    } else {
        format!("{}  │   ", prefix)
    };

    if let Some(deps) = children.get(name) {
        for (j, dep) in deps.iter().enumerate() {
            let is_last_child = j == deps.len() - 1;
            render_tree_node(dep, &child_prefix, is_last_child, children, affected, visited);
        }
    }
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
    format: CiFormat,
) -> Result<()> {
    let result = affected_core::find_affected_with_options(root, base, false, filter, skip)?;

    let affected_csv = result.affected.join(",");
    let count = result.affected.len();
    let has_affected = !result.affected.is_empty();
    let affected_json = serde_json::to_string(&result.affected)?;
    let matrix = serde_json::json!({"package": &result.affected});
    let matrix_str = serde_json::to_string(&matrix)?;

    match format {
        CiFormat::Github => {
            if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
                let mut content = String::new();
                content.push_str(&format!("affected={}\n", affected_csv));
                content.push_str(&format!("count={}\n", count));
                content.push_str(&format!("has_affected={}\n", has_affected));
                content.push_str(&format!("affected_json={}\n", affected_json));
                content.push_str(&format!("matrix={}\n", matrix_str));
                std::fs::write(output_file, content)?;
            }
            println!("affected={}", affected_csv);
            println!("count={}", count);
            println!("has_affected={}", has_affected);
            println!("matrix={}", matrix_str);
        }
        CiFormat::Gitlab => {
            // Write dotenv file for GitLab CI artifacts
            let dotenv = format!(
                "AFFECTED={}\nAFFECTED_COUNT={}\nHAS_AFFECTED={}\nAFFECTED_JSON={}\nAFFECTED_MATRIX={}\n",
                affected_csv, count, has_affected, affected_json, matrix_str
            );
            std::fs::write("ci.env", &dotenv)?;
            // Also print for stdout capture
            print!("{}", dotenv);
        }
        CiFormat::Circleci => {
            if let Ok(bash_env) = std::env::var("BASH_ENV") {
                let content = format!(
                    "export AFFECTED=\"{}\"\nexport AFFECTED_COUNT=\"{}\"\nexport HAS_AFFECTED=\"{}\"\nexport AFFECTED_JSON='{}'\nexport AFFECTED_MATRIX='{}'\n",
                    affected_csv, count, has_affected, affected_json, matrix_str
                );
                // Append to BASH_ENV
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(bash_env)?;
                file.write_all(content.as_bytes())?;
            }
            println!("affected={}", affected_csv);
            println!("count={}", count);
            println!("has_affected={}", has_affected);
            println!("matrix={}", matrix_str);
        }
        CiFormat::Azure => {
            println!(
                "##vso[task.setvariable variable=affected;isOutput=true]{}",
                affected_csv
            );
            println!(
                "##vso[task.setvariable variable=affected_count;isOutput=true]{}",
                count
            );
            println!(
                "##vso[task.setvariable variable=has_affected;isOutput=true]{}",
                has_affected
            );
            println!(
                "##vso[task.setvariable variable=affected_json;isOutput=true]{}",
                affected_json
            );
            println!(
                "##vso[task.setvariable variable=matrix;isOutput=true]{}",
                matrix_str
            );
        }
        CiFormat::Generic => {
            println!("affected={}", affected_csv);
            println!("count={}", count);
            println!("has_affected={}", has_affected);
            println!("affected_json={}", affected_json);
            println!("matrix={}", matrix_str);
        }
    }

    Ok(())
}

fn cmd_init(root: &std::path::Path, non_interactive: bool) -> Result<()> {
    let config_path = root.join(".affected.toml");

    if config_path.exists() && !non_interactive {
        let overwrite = dialoguer::Confirm::new()
            .with_prompt(".affected.toml already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            println!("{}", "Aborted.".dimmed());
            return Ok(());
        }
    }

    // Auto-detect ecosystems
    let detected = affected_core::detect::detect_ecosystems(root).unwrap_or_default();
    let ecosystem_name = detected.first().map(|e| e.to_string());

    if !non_interactive {
        if let Some(ref eco) = ecosystem_name {
            println!(
                "  {} Detected ecosystem: {}",
                "✓".green(),
                eco.cyan()
            );
        } else {
            println!(
                "  {} No ecosystem auto-detected. Config will use defaults.",
                "!".yellow()
            );
        }
    }

    let mut test_section = String::new();
    let mut ignore_patterns: Vec<String> =
        vec!["*.md".into(), "docs/**".into(), ".github/**".into()];

    if !non_interactive {
        // Ask for custom test command
        let custom_test: String = dialoguer::Input::new()
            .with_prompt("Custom test command (leave blank for default)")
            .allow_empty(true)
            .interact_text()?;

        if !custom_test.is_empty() {
            if let Some(ref eco) = ecosystem_name {
                test_section = format!("{} = \"{}\"", eco, custom_test);
            }
        }

        // Ask for ignore patterns
        let custom_ignore: String = dialoguer::Input::new()
            .with_prompt("Ignore patterns (comma-separated, leave blank for defaults)")
            .allow_empty(true)
            .interact_text()?;

        if !custom_ignore.is_empty() {
            ignore_patterns = custom_ignore
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
    }

    // Generate config
    let ignore_str = ignore_patterns
        .iter()
        .map(|p| format!("\"{}\"", p))
        .collect::<Vec<_>>()
        .join(", ");

    let mut config_content = String::from(
        "# .affected.toml — configuration for the `affected` CLI\n\n",
    );

    config_content.push_str(&format!("ignore = [{}]\n\n", ignore_str));
    config_content.push_str("[test]\n");
    if test_section.is_empty() {
        config_content.push_str("# Uncomment and customize test commands per ecosystem:\n");
        config_content.push_str("# cargo = \"cargo nextest run -p {package}\"\n");
        config_content.push_str("# npm = \"pnpm run test --filter {package}\"\n");
        config_content.push_str("# go = \"go test ./{package}/...\"\n");
        config_content.push_str("# python = \"python -m pytest {package}\"\n");
        config_content.push_str("# bun = \"bun test --filter {package}\"\n");
        config_content.push_str("# dotnet = \"dotnet test {package}\"\n");
        config_content.push_str("# dart = \"dart test -C {package}\"\n");
        config_content.push_str("# swift = \"swift test --filter {package}\"\n");
        config_content.push_str("# elixir = \"mix cmd --app {package} mix test\"\n");
        config_content.push_str("# sbt = \"sbt {package}/test\"\n");
    } else {
        config_content.push_str(&format!("{}\n", test_section));
    }

    std::fs::write(&config_path, config_content)?;
    println!(
        "\n  {} Created {}",
        "✓".green().bold(),
        config_path.display().to_string().cyan(),
    );

    Ok(())
}

fn cmd_watch(
    root: &std::path::Path,
    subcommand: WatchableCommand,
    debounce_ms: u64,
    config_path: Option<&std::path::Path>,
    quiet: bool,
) -> Result<()> {
    use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
    use std::sync::mpsc;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel();
    let debounce_dur = Duration::from_millis(debounce_ms);
    let mut debouncer = new_debouncer(debounce_dur, tx)?;

    // Watch the project root recursively
    debouncer.watcher().watch(
        root,
        notify::RecursiveMode::Recursive,
    )?;

    // Directories to ignore
    let ignore_dirs: Vec<&str> = vec![".git", "target", "node_modules", "_build", ".dart_tool"];

    println!(
        "{} Watching for changes... (Ctrl+C to stop)\n",
        "👀".to_string().bold()
    );

    // Run once immediately
    let _ = run_watchable(&subcommand, root, config_path, quiet);

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Filter out events in ignored directories
                let relevant = events.iter().any(|event| {
                    if event.kind != DebouncedEventKind::Any {
                        return false;
                    }
                    let path_str = event.path.to_string_lossy();
                    !ignore_dirs.iter().any(|d| path_str.contains(&format!("/{}/", d)))
                });

                if relevant {
                    println!(
                        "\n{} Change detected, re-running...\n",
                        "↻".cyan().bold()
                    );
                    let _ = run_watchable(&subcommand, root, config_path, quiet);
                    println!(
                        "\n{} Watching for changes...",
                        "👀".to_string().dimmed()
                    );
                }
            }
            Ok(Err(error)) => {
                eprintln!("Watch error: {}", error);
            }
            Err(_) => break,
        }
    }

    Ok(())
}

fn run_watchable(
    subcommand: &WatchableCommand,
    root: &std::path::Path,
    config_path: Option<&std::path::Path>,
    quiet: bool,
) -> Result<()> {
    match subcommand {
        WatchableCommand::Test {
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
            let base_ref = resolve_base(root, base.clone(), merge_base.clone())?;
            cmd_test(
                root,
                &base_ref,
                *dry_run,
                *json,
                filter.as_deref(),
                skip.as_deref(),
                *explain,
                *jobs,
                *timeout,
                None,
                config_path,
                quiet,
            )
        }
        WatchableCommand::List {
            base,
            merge_base,
            json,
            filter,
            skip,
            explain,
        } => {
            let base_ref = resolve_base(root, base.clone(), merge_base.clone())?;
            cmd_list(
                root,
                &base_ref,
                *json,
                filter.as_deref(),
                skip.as_deref(),
                *explain,
                quiet,
            )
        }
        WatchableCommand::Run {
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
            let base_ref = resolve_base(root, base.clone(), merge_base.clone())?;
            cmd_run(
                root,
                &base_ref,
                command,
                *dry_run,
                *json,
                filter.as_deref(),
                skip.as_deref(),
                *explain,
                *jobs,
                *timeout,
                quiet,
            )
        }
    }
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
