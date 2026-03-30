use anyhow::Result;
use std::path::Path;
use tracing::debug;

use crate::types::Ecosystem;

/// Detect which ecosystem(s) a project uses by scanning for marker files.
pub fn detect_ecosystems(root: &Path) -> Result<Vec<Ecosystem>> {
    let mut detected = Vec::new();

    // Cargo: Cargo.toml with [workspace]
    let cargo_toml = root.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            if content.contains("[workspace]") {
                debug!("Detected Cargo workspace at {}", cargo_toml.display());
                detected.push(Ecosystem::Cargo);
            }
        }
    }

    // Yarn: .yarnrc.yml exists → Ecosystem::Yarn (takes priority over Bun/npm)
    let yarnrc = root.join(".yarnrc.yml");
    if yarnrc.exists() {
        debug!("Detected Yarn Berry project via .yarnrc.yml");
        detected.push(Ecosystem::Yarn);
    } else {
        // Bun: bun.lock/bun.lockb/bunfig.toml (takes priority over npm)
        let has_bun = root.join("bun.lock").exists()
            || root.join("bun.lockb").exists()
            || root.join("bunfig.toml").exists();
        let pkg_json = root.join("package.json");
        let pnpm_ws = root.join("pnpm-workspace.yaml");

        let has_workspaces = if pnpm_ws.exists() {
            true
        } else if pkg_json.exists() {
            std::fs::read_to_string(&pkg_json)
                .map(|c| c.contains("\"workspaces\""))
                .unwrap_or(false)
        } else {
            false
        };

        if has_bun && has_workspaces {
            debug!("Detected Bun workspace via bun.lock/bunfig.toml");
            detected.push(Ecosystem::Bun);
        } else if pnpm_ws.exists() {
            debug!("Detected pnpm workspace via pnpm-workspace.yaml");
            detected.push(Ecosystem::Npm);
        } else if pkg_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                if content.contains("\"workspaces\"") {
                    debug!("Detected npm workspaces in package.json");
                    detected.push(Ecosystem::Npm);
                }
            }
        }
    }

    // Go: go.work (workspace) or go.mod (single module)
    if root.join("go.work").exists() || root.join("go.mod").exists() {
        debug!("Detected Go project");
        detected.push(Ecosystem::Go);
    }

    // Python: check for Poetry, uv, or generic pyproject.toml
    let root_pyproject = root.join("pyproject.toml");
    if root_pyproject.exists() {
        if let Ok(content) = std::fs::read_to_string(&root_pyproject) {
            if content.contains("[tool.poetry]") {
                debug!("Detected Poetry project via [tool.poetry] in pyproject.toml");
                detected.push(Ecosystem::Python);
            } else if content.contains("[tool.uv.workspace]") {
                debug!("Detected uv workspace via [tool.uv.workspace] in pyproject.toml");
                detected.push(Ecosystem::Python);
            } else {
                debug!("Detected generic Python project via pyproject.toml");
                detected.push(Ecosystem::Python);
            }
        } else {
            detected.push(Ecosystem::Python);
        }
    } else {
        // Scan one level deep for pyproject.toml files
        let pattern = root.join("*/pyproject.toml");
        if let Ok(paths) = glob::glob(pattern.to_str().unwrap_or("")) {
            let count = paths.filter_map(|p| p.ok()).count();
            if count >= 2 {
                debug!(
                    "Detected Python monorepo ({} pyproject.toml files found)",
                    count
                );
                detected.push(Ecosystem::Python);
            }
        }
    }

    // Maven: pom.xml exists at root and contains <modules>
    let pom_xml = root.join("pom.xml");
    if pom_xml.exists() {
        if let Ok(content) = std::fs::read_to_string(&pom_xml) {
            if content.contains("<modules>") {
                debug!("Detected Maven multi-module project via pom.xml");
                detected.push(Ecosystem::Maven);
            }
        }
    }

    // Gradle: settings.gradle or settings.gradle.kts exists
    if root.join("settings.gradle").exists() || root.join("settings.gradle.kts").exists() {
        debug!("Detected Gradle project");
        detected.push(Ecosystem::Gradle);
    }

    // .NET: *.sln file at root
    let sln_pattern = root.join("*.sln");
    if let Ok(mut paths) = glob::glob(sln_pattern.to_str().unwrap_or("")) {
        if paths.any(|p| p.is_ok()) {
            debug!("Detected .NET solution via *.sln");
            detected.push(Ecosystem::Dotnet);
        }
    }

    // Swift: Package.swift with multiple targets or multiple Package.swift in subdirs
    let package_swift = root.join("Package.swift");
    if package_swift.exists() {
        if let Ok(content) = std::fs::read_to_string(&package_swift) {
            let target_count = content.matches(".target(").count()
                + content.matches(".executableTarget(").count()
                + content.matches(".testTarget(").count();
            if target_count >= 2 {
                debug!("Detected Swift package with {} targets", target_count);
                detected.push(Ecosystem::Swift);
            }
        }
    }

    // Dart/Flutter: pubspec.yaml with workspace, melos.yaml, or multiple pubspec.yaml
    let root_pubspec = root.join("pubspec.yaml");
    if root_pubspec.exists() {
        if let Ok(content) = std::fs::read_to_string(&root_pubspec) {
            if content.contains("workspace:") {
                debug!("Detected Dart workspace via pubspec.yaml workspace field");
                detected.push(Ecosystem::Dart);
            }
        }
    }
    if !detected.contains(&Ecosystem::Dart) && root.join("melos.yaml").exists() {
        debug!("Detected Dart/Flutter monorepo via melos.yaml");
        detected.push(Ecosystem::Dart);
    }
    if !detected.contains(&Ecosystem::Dart) {
        let pattern = root.join("*/pubspec.yaml");
        if let Ok(paths) = glob::glob(pattern.to_str().unwrap_or("")) {
            let count = paths.filter_map(|p| p.ok()).count();
            if count >= 2 {
                debug!(
                    "Detected Dart monorepo ({} pubspec.yaml files found)",
                    count
                );
                detected.push(Ecosystem::Dart);
            }
        }
    }

    // Elixir: mix.exs + apps/ directory (umbrella project)
    if root.join("mix.exs").exists() && root.join("apps").is_dir() {
        debug!("Detected Elixir umbrella project via mix.exs + apps/");
        detected.push(Ecosystem::Elixir);
    }

    // Scala/sbt: build.sbt at root
    if root.join("build.sbt").exists() {
        debug!("Detected sbt project via build.sbt");
        detected.push(Ecosystem::Sbt);
    }

    if detected.is_empty() {
        anyhow::bail!(
            "No supported project type found at {}.\n\
             Looked for: Cargo.toml (workspace), package.json (workspaces), \
             go.work/go.mod, pyproject.toml, pom.xml (modules), settings.gradle(.kts), \
             *.sln (.NET), Package.swift, pubspec.yaml/melos.yaml, mix.exs (umbrella), build.sbt",
            root.display()
        );
    }

    debug!("Detected ecosystems: {:?}", detected);
    Ok(detected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cargo_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Cargo]);
    }

    #[test]
    fn test_detect_cargo_without_workspace_ignored() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"solo\"\n",
        )
        .unwrap();

        assert!(detect_ecosystems(dir.path()).is_err());
    }

    #[test]
    fn test_detect_npm_workspaces() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "root", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Npm]);
    }

    #[test]
    fn test_detect_pnpm_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pnpm-workspace.yaml"),
            "packages:\n  - 'packages/*'\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Npm]);
    }

    #[test]
    fn test_detect_yarn_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".yarnrc.yml"), "nodeLinker: pnp\n").unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Yarn]);
    }

    #[test]
    fn test_detect_go_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("go.work"), "go 1.21\n").unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Go]);
    }

    #[test]
    fn test_detect_go_single_module() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("go.mod"), "module example.com/foo\n").unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Go]);
    }

    #[test]
    fn test_detect_python_root_pyproject() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[project]\nname = \"myapp\"\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Python]);
    }

    #[test]
    fn test_detect_multiple_ecosystems() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        std::fs::write(dir.path().join("go.mod"), "module example.com/x\n").unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Cargo));
        assert!(ecosystems.contains(&Ecosystem::Go));
        assert_eq!(ecosystems.len(), 2);
    }

    #[test]
    fn test_detect_empty_directory_errors() {
        let dir = tempfile::tempdir().unwrap();
        assert!(detect_ecosystems(dir.path()).is_err());
    }

    #[test]
    fn test_detect_npm_without_workspaces_ignored() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "solo", "version": "1.0.0"}"#,
        )
        .unwrap();

        assert!(detect_ecosystems(dir.path()).is_err());
    }

    #[test]
    fn test_detect_maven_multi_module() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pom.xml"),
            r#"<project><modules><module>core</module></modules></project>"#,
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Maven]);
    }

    #[test]
    fn test_detect_gradle_groovy() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.gradle"),
            "include ':core', ':app'\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Gradle]);
    }

    #[test]
    fn test_detect_gradle_kotlin() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.gradle.kts"),
            "include(\":core\", \":app\")\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Gradle]);
    }

    #[test]
    fn test_detect_poetry_project() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[tool.poetry]\nname = \"myapp\"\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Python]);
    }

    #[test]
    fn test_detect_uv_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[tool.uv.workspace]\nmembers = [\"packages/*\"]\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert_eq!(ecosystems, vec![Ecosystem::Python]);
    }

    #[test]
    fn test_detect_bun_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bun.lock"), "").unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "root", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Bun));
    }

    #[test]
    fn test_detect_bun_lockb() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bun.lockb"), "").unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "root", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Bun));
    }

    #[test]
    fn test_detect_dotnet_solution() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("MySolution.sln"), "Microsoft Visual Studio Solution File").unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Dotnet));
    }

    #[test]
    fn test_detect_swift_package() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Package.swift"),
            r#"let package = Package(
    name: "MyPkg",
    targets: [
        .target(name: "Core", dependencies: []),
        .target(name: "API", dependencies: ["Core"]),
    ]
)"#,
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Swift));
    }

    #[test]
    fn test_detect_dart_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pubspec.yaml"),
            "name: root\nworkspace:\n  - packages/core\n  - packages/api\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Dart));
    }

    #[test]
    fn test_detect_melos() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("melos.yaml"),
            "name: my_project\npackages:\n  - packages/*\n",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Dart));
    }

    #[test]
    fn test_detect_elixir_umbrella() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("mix.exs"), "defmodule Root.MixProject do\nend").unwrap();
        std::fs::create_dir_all(dir.path().join("apps")).unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Elixir));
    }

    #[test]
    fn test_detect_sbt_project() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("build.sbt"),
            "lazy val root = (project in file(\".\"))",
        )
        .unwrap();

        let ecosystems = detect_ecosystems(dir.path()).unwrap();
        assert!(ecosystems.contains(&Ecosystem::Sbt));
    }
}
