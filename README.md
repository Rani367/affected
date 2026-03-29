<p align="center">
  <h1 align="center">affected</h1>
  <p align="center">Run only the tests that matter.</p>
</p>

<p align="center">
  <a href="https://github.com/Rani367/affected/actions/workflows/ci.yml"><img src="https://github.com/Rani367/affected/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/affected-cli"><img src="https://img.shields.io/crates/v/affected-cli.svg" alt="crates.io"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://github.com/Rani367/affected/stargazers"><img src="https://img.shields.io/github/stars/Rani367/affected?style=social" alt="GitHub Stars"></a>
</p>

<p align="center">
  A standalone, language-agnostic CLI that detects which packages in your monorepo are affected by git changes and runs only their tests. No framework, no config files, no lock-in.
</p>

---

## Demo

```
$ affected list --base main --explain

3 affected package(s) (base: main, 2 files changed):

  ● core       (directly changed: src/lib.rs)
  ● api        (depends on: core)
  ● cli        (depends on: api → core)
```

```
$ affected test --base main --jobs 4 --dry-run

Testing 3 affected package(s) (out of 8 total, 2 files changed):

  [dry-run] core: cargo test -p core
  [dry-run] api: cargo test -p api
  [dry-run] cli: cargo test -p cli

  Results: 3 passed, 0 failed, 3 total (0.0s)
```

## Why

Every monorepo team hacks together bash scripts with `git diff | grep` to avoid running all tests on every PR. Tools like Nx, Turborepo, and Bazel solve this but require buying into an entire build system.

`affected` is a single binary you `cargo install` and run. It auto-detects your project type, builds a dependency graph, and figures out what to test. Zero config to start.

## Features

- **Zero config** -- auto-detects your project type and dependency graph
- **7 ecosystems** -- Cargo, npm, pnpm, Yarn Berry, Go, Python (Poetry/uv), Maven, Gradle
- **Transitive detection** -- if `core` changes and `api` depends on `core`, both are tested
- **`--explain`** -- shows *why* each package is affected with the full dependency chain
- **Parallel tests** -- `--jobs 4` runs tests across multiple threads
- **CI-first** -- `--json`, `--junit`, and `affected ci` for GitHub Actions integration
- **Fast** -- written in Rust, uses libgit2 for native git operations

## Install

```bash
cargo install affected-cli
```

Or download a pre-built binary from [Releases](https://github.com/Rani367/affected/releases).

## Quick Start

```bash
# What's affected?
affected list --base main

# Run only affected tests
affected test --base main

# See why each package is affected
affected list --base main --explain

# Dry run (show commands without executing)
affected test --base main --dry-run

# Parallel execution with 4 threads
affected test --base main --jobs 4
```

## Usage

### `affected test`

Run tests for affected packages.

```bash
affected test --base main                     # run affected tests
affected test --base HEAD~3                   # compare vs 3 commits ago
affected test --merge-base main               # auto-detect merge-base (best for PRs)
affected test --base main --jobs 4            # parallel execution
affected test --base main --timeout 300       # 5 min timeout per package
affected test --base main --dry-run           # show what would run
affected test --base main --json              # structured JSON output
affected test --base main --junit results.xml # JUnit XML for CI
affected test --base main --filter "lib-*"    # only test matching packages
affected test --base main --skip "e2e-*"      # skip matching packages
affected test --base main --explain           # show why each is affected
```

### `affected list`

List affected packages without running tests.

```bash
affected list --base main                     # list affected packages
affected list --base main --json              # JSON output for CI
affected list --base main --explain           # show dependency chains
```

### `affected graph`

Display the project dependency graph.

```bash
affected graph                                # human-readable graph
affected graph --dot                          # DOT format for Graphviz
affected graph --dot | dot -Tpng -o graph.png # render as image
```

### `affected ci`

Output variables for CI systems (GitHub Actions).

```bash
affected ci --base main
# Output:
#   affected=core,api,cli
#   count=3
#   has_affected=true
```

#### GitHub Actions Example

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # needed for git diff

      - name: Install affected
        run: cargo install affected-cli

      - name: Detect affected packages
        id: affected
        run: affected ci --merge-base main

      - name: Run affected tests
        if: steps.affected.outputs.has_affected == 'true'
        run: affected test --merge-base main --jobs 4 --junit results.xml
```

### `affected completions`

Generate shell completions.

```bash
affected completions bash >> ~/.bashrc
affected completions zsh >> ~/.zshrc
affected completions fish > ~/.config/fish/completions/affected.fish
```

## Supported Ecosystems

| Ecosystem | Detected By | Dependency Source |
|-----------|------------|-------------------|
| **Cargo** | `Cargo.toml` with `[workspace]` | `cargo metadata` JSON |
| **npm** | `package.json` with `workspaces` | `package.json` dependencies |
| **pnpm** | `pnpm-workspace.yaml` | `package.json` dependencies |
| **Yarn Berry** | `.yarnrc.yml` | `package.json` dependencies |
| **Go** | `go.work` / `go.mod` | `go mod graph` |
| **Python** | `pyproject.toml` | PEP 621 deps + import scanning |
| **Poetry** | `[tool.poetry]` in pyproject.toml | Poetry path dependencies |
| **uv** | `[tool.uv.workspace]` in pyproject.toml | Workspace member globs |
| **Maven** | `pom.xml` with `<modules>` | POM dependency declarations |
| **Gradle** | `settings.gradle(.kts)` | `project(':...')` references |

## Configuration

Create `.affected.toml` in your project root (optional):

```toml
# Ignore files that should never trigger tests
ignore = ["*.md", "docs/**", ".github/**"]

# Custom test commands per ecosystem
[test]
cargo = "cargo nextest run -p {package}"
npm = "pnpm test --filter {package}"
go = "go test -v ./{package}/..."
python = "uv run --package {package} pytest"
maven = "mvn test -pl {package}"
gradle = "gradle :{package}:test"

# Per-package overrides
[packages.slow-e2e]
test = "cargo test -p slow-e2e -- --ignored"
timeout = 600

[packages.legacy-service]
skip = true
```

## How It Works

1. **Detect** -- scans for marker files to identify the ecosystem
2. **Resolve** -- builds a dependency graph from project manifests
3. **Diff** -- computes changed files using libgit2 (base ref vs HEAD + working tree)
4. **Map** -- maps each changed file to its owning package
5. **Traverse** -- runs reverse BFS on the dependency graph to find all transitively affected packages
6. **Execute** -- runs test commands for affected packages only

## Comparison

| Feature | `affected` | Nx | Turborepo | Bazel |
|---------|-----------|-----|-----------|-------|
| Zero config | Yes | No | No | No |
| Standalone binary | Yes | No (Node.js) | No (Node.js) | No (JVM) |
| Language agnostic | 7 ecosystems | JS/TS + plugins | JS/TS | Any (with rules) |
| Setup time | 1 minute | Hours | Hours | Days-weeks |
| `--explain` | Yes | No | No | No |
| Binary size | ~5MB | ~200MB+ | ~100MB+ | ~500MB+ |

## Global Flags

```
-v, --verbose    Increase verbosity (-v for debug, -vv for trace)
-q, --quiet      Suppress non-essential output
--no-color       Disable colored output (also respects NO_COLOR env var)
--root <PATH>    Path to project root (default: current directory)
--config <PATH>  Path to custom config file
```

## Contributing

Contributions welcome! See [issues](https://github.com/Rani367/affected/issues) for ideas, or open a PR.

## License

MIT

---

If this tool saves you CI time, consider giving it a star. It helps others find it.
