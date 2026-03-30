<p align="center">
  <h1 align="center">affected</h1>
  <p align="center">Detect affected packages. Run only what matters.</p>
</p>

<p align="center">
  <a href="https://github.com/Rani367/affected/actions/workflows/ci.yml"><img src="https://github.com/Rani367/affected/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/affected-cli"><img src="https://img.shields.io/crates/v/affected-cli.svg" alt="crates.io"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://github.com/Rani367/affected/stargazers"><img src="https://img.shields.io/github/stars/Rani367/affected?style=social" alt="GitHub Stars"></a>
</p>

<p align="center">
  A standalone, language-agnostic CLI that detects which packages in your monorepo are affected by git changes — then runs tests, lints, builds, or any command on only those packages. No framework, no config files, no lock-in.
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
$ affected run "cargo clippy -p {package}" --base main --jobs 4 --dry-run

Running command for 3 affected package(s) (out of 8 total, 2 files changed):

  [dry-run] core: cargo clippy -p core
  [dry-run] api: cargo clippy -p api
  [dry-run] cli: cargo clippy -p cli
```

## Why

Every monorepo team hacks together bash scripts with `git diff | grep` to avoid running all tests on every PR. Tools like Nx, Turborepo, and Bazel solve this but require buying into an entire build system.

`affected` is a single binary you install and run. It auto-detects your project type, builds a dependency graph, and figures out what's affected. Zero config to start.

## Features

- **Zero config** -- auto-detects your project type and dependency graph
- **13 ecosystems** -- Cargo, npm, pnpm, Yarn, Bun, Go, Python, Maven, Gradle, .NET, Swift, Dart/Flutter, Elixir, Scala/sbt
- **Transitive detection** -- if `core` changes and `api` depends on `core`, both are affected
- **`affected run`** -- run *any command* on affected packages, not just tests
- **`--explain`** -- shows *why* each package is affected with the full dependency chain
- **Multi-CI** -- `affected ci --format github|gitlab|circleci|azure` with dynamic job matrices
- **Watch mode** -- `affected watch test --base main` re-runs on file changes
- **`affected init`** -- interactive setup wizard generates `.affected.toml`
- **Dependency tree** -- `affected graph` renders a Unicode tree with affected highlighting
- **Parallel execution** -- `--jobs 4` runs commands across multiple threads
- **CI-first** -- `--json`, `--junit`, PR comment bot, shell completions
- **Fast** -- written in Rust, uses libgit2 for native git operations

## Install

```bash
# Homebrew (macOS/Linux)
brew install Rani367/tap/affected

# uv
uv tool install affected

# pipx
pipx install affected

# pip
pip install affected

# Cargo
cargo install affected-cli

# GitHub Actions
- uses: Rani367/setup-affected@v1

# Or download a binary from Releases
```

## Quick Start

```bash
# What's affected?
affected list --base main

# See why each package is affected
affected list --base main --explain

# Run only affected tests
affected test --base main

# Run any command on affected packages
affected run "cargo clippy -p {package}" --base main

# Parallel execution
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

### `affected run`

Run any command on affected packages. Use `{package}` as a placeholder.

```bash
affected run "cargo clippy -p {package}" --base main         # lint affected
affected run "cargo build -p {package}" --base main --jobs 4  # build affected
affected run "npm run lint --workspace={package}" --base main  # npm lint
affected run "go vet ./{package}/..." --base main              # go vet
affected run "echo {package}" --base main --dry-run            # preview
```

### `affected list`

List affected packages without running anything.

```bash
affected list --base main                     # list affected packages
affected list --base main --json              # JSON output for CI
affected list --base main --explain           # show dependency chains
```

### `affected graph`

Display the project dependency graph as a Unicode tree.

```bash
affected graph                                # Unicode dependency tree
affected graph --base main                    # highlight affected packages
affected graph --dot                          # DOT format for Graphviz
affected graph --dot | dot -Tpng -o graph.png # render as image
```

Example output:

```
Dependency Graph (5 packages, 3 affected):

  cli  ●
  └── api  ●
      └── core  ●
  utils
  standalone  (no dependencies)
```

### `affected ci`

Output variables for CI systems with multi-platform support.

```bash
affected ci --base main                        # GitHub Actions (default)
affected ci --base main --format gitlab        # GitLab CI (writes ci.env)
affected ci --base main --format azure         # Azure Pipelines (##vso)
affected ci --base main --format circleci      # CircleCI ($BASH_ENV)
affected ci --base main --format generic       # plain key=value
```

### `affected init`

Interactive setup wizard to generate `.affected.toml`.

```bash
affected init                                  # interactive prompts
affected init --non-interactive                # auto-detect and use defaults
```

### `affected watch`

Watch for file changes and re-run commands automatically.

```bash
affected watch test --base main                # re-run tests on changes
affected watch list --base main                # re-list affected on changes
affected watch run "cargo clippy -p {package}" --base main  # re-run command
affected watch test --base main --debounce 1000 # 1s debounce
```

## GitHub Actions

### Setup

```yaml
- uses: Rani367/setup-affected@v1
- run: affected test --merge-base origin/main
```

### Dynamic Matrix (each package as a separate job)

Each affected package runs as a **separate parallel job** in the GitHub Actions UI:

```yaml
jobs:
  detect:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.affected.outputs.matrix }}
      has_affected: ${{ steps.affected.outputs.has_affected }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: Rani367/setup-affected@v1
      - id: affected
        run: affected ci --merge-base origin/main

  test:
    needs: detect
    if: needs.detect.outputs.has_affected == 'true'
    runs-on: ubuntu-latest
    strategy:
      matrix: ${{ fromJson(needs.detect.outputs.matrix) }}
      fail-fast: false
    steps:
      - uses: actions/checkout@v4
      - name: Test ${{ matrix.package }}
        run: cargo test -p ${{ matrix.package }}
```

### PR Comment Bot

Auto-comment on PRs showing which packages are affected and why:

```yaml
on:
  pull_request:
    branches: [main]

permissions:
  pull-requests: write

jobs:
  comment:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: Rani367/affected-pr-comment@v1
```

This posts a comment like:

> **Affected Packages (3 of 8)**
>
> | Package | Reason |
> |---------|--------|
> | **core** | directly changed: `src/lib.rs` |
> | **api** | depends on: core |
> | **cli** | depends on: api -> core |

The comment updates automatically on each push (no duplicates).

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
| **Bun** | `bun.lock` / `bunfig.toml` | `package.json` dependencies |
| **Go** | `go.work` / `go.mod` | `go mod graph` |
| **Python** | `pyproject.toml` | PEP 621 deps + import scanning |
| **Poetry** | `[tool.poetry]` in pyproject.toml | Poetry path dependencies |
| **uv** | `[tool.uv.workspace]` in pyproject.toml | Workspace member globs |
| **Maven** | `pom.xml` with `<modules>` | POM dependency declarations |
| **Gradle** | `settings.gradle(.kts)` | `project(':...')` references |
| **.NET/C#** | `*.sln` solution file | `<ProjectReference>` in .csproj |
| **Swift/SPM** | `Package.swift` (multi-target) | Target dependency declarations |
| **Dart/Flutter** | `pubspec.yaml` workspace / `melos.yaml` | `dependencies` in pubspec.yaml |
| **Elixir** | `mix.exs` + `apps/` (umbrella) | `in_umbrella: true` deps |
| **Scala/sbt** | `build.sbt` | `.dependsOn()` project refs |

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
bun = "bun test --filter {package}"
dotnet = "dotnet test {package}"
dart = "dart test -C {package}"
swift = "swift test --filter {package}"
elixir = "mix cmd --app {package} mix test"
sbt = "sbt {package}/test"

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
6. **Execute** -- runs commands for affected packages only

## Comparison

| Feature | `affected` | Nx | Turborepo | Bazel |
|---------|-----------|-----|-----------|-------|
| Zero config | Yes | No | No | No |
| Standalone binary | Yes | No (Node.js) | No (Node.js) | No (JVM) |
| Language agnostic | 13 ecosystems | JS/TS + plugins | JS/TS | Any (with rules) |
| Setup time | 1 minute | Hours | Hours | Days-weeks |
| `affected run <cmd>` | Yes | No | No | No |
| `--explain` | Yes | No | No | No |
| Watch mode | Yes | Yes | No | No |
| Multi-CI support | 5 platforms | GitHub only | GitHub only | Custom |
| Dynamic CI matrix | Yes | Plugin | No | No |
| PR comment bot | Yes | No | No | No |
| Interactive setup | `affected init` | `nx init` | `turbo init` | Manual |
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
