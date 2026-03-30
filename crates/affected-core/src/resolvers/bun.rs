use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::resolvers::{file_to_package, Resolver};
use crate::types::{Ecosystem, Package, PackageId, ProjectGraph};

pub struct BunResolver;

#[derive(Deserialize)]
struct RootPackageJson {
    workspaces: Option<WorkspacesField>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum WorkspacesField {
    Array(Vec<String>),
    Object { packages: Vec<String> },
}

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

impl Resolver for BunResolver {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Bun
    }

    fn detect(&self, root: &Path) -> bool {
        // Bun uses the same package.json workspaces as npm, but is identified by bun.lock/bunfig.toml
        let has_bun_marker = root.join("bun.lock").exists()
            || root.join("bun.lockb").exists()
            || root.join("bunfig.toml").exists();

        if !has_bun_marker {
            return false;
        }

        // Also need workspaces configured
        if root.join("pnpm-workspace.yaml").exists() {
            return true;
        }
        let pkg = root.join("package.json");
        if !pkg.exists() {
            return false;
        }
        std::fs::read_to_string(&pkg)
            .map(|c| c.contains("\"workspaces\""))
            .unwrap_or(false)
    }

    fn resolve(&self, root: &Path) -> Result<ProjectGraph> {
        let workspace_globs = find_workspace_globs(root)?;
        let pkg_dirs = expand_globs(root, &workspace_globs)?;

        let mut packages = HashMap::new();
        let mut name_to_id = HashMap::new();

        for dir in &pkg_dirs {
            let pkg_json_path = dir.join("package.json");
            if !pkg_json_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&pkg_json_path)
                .with_context(|| format!("Failed to read {}", pkg_json_path.display()))?;
            let pkg: PackageJson = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", pkg_json_path.display()))?;

            let name = match &pkg.name {
                Some(n) => n.clone(),
                None => continue,
            };

            let pkg_id = PackageId(name.clone());
            name_to_id.insert(name.clone(), pkg_id.clone());
            packages.insert(
                pkg_id.clone(),
                Package {
                    id: pkg_id,
                    name: name.clone(),
                    version: pkg.version.clone(),
                    path: dir.clone(),
                    manifest_path: pkg_json_path,
                },
            );
        }

        // Build dependency edges
        let mut edges = Vec::new();
        let workspace_names: std::collections::HashSet<&str> =
            name_to_id.keys().map(|s| s.as_str()).collect();

        for dir in &pkg_dirs {
            let pkg_json_path = dir.join("package.json");
            if !pkg_json_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&pkg_json_path)?;
            let pkg: PackageJson = serde_json::from_str(&content)?;

            let from_name = match &pkg.name {
                Some(n) => n.clone(),
                None => continue,
            };

            let all_deps: Vec<&str> = pkg
                .dependencies
                .iter()
                .flat_map(|d| d.keys())
                .chain(pkg.dev_dependencies.iter().flat_map(|d| d.keys()))
                .map(|s| s.as_str())
                .collect();

            for dep_name in all_deps {
                if workspace_names.contains(dep_name) {
                    edges.push((
                        PackageId(from_name.clone()),
                        PackageId(dep_name.to_string()),
                    ));
                }
            }
        }

        Ok(ProjectGraph {
            packages,
            edges,
            root: root.to_path_buf(),
        })
    }

    fn package_for_file(&self, graph: &ProjectGraph, file: &Path) -> Option<PackageId> {
        file_to_package(graph, file)
    }

    fn test_command(&self, package_id: &PackageId) -> Vec<String> {
        vec![
            "bun".into(),
            "test".into(),
            "--filter".into(),
            package_id.0.clone(),
        ]
    }
}

fn find_workspace_globs(root: &Path) -> Result<Vec<String>> {
    // Try pnpm-workspace.yaml first (Bun supports it)
    let pnpm_path = root.join("pnpm-workspace.yaml");
    if pnpm_path.exists() {
        let content = std::fs::read_to_string(&pnpm_path)?;
        let mut globs = Vec::new();
        let mut in_packages = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "packages:" {
                in_packages = true;
                continue;
            }
            if in_packages {
                if trimmed.starts_with("- ") {
                    let glob = trimmed
                        .trim_start_matches("- ")
                        .trim_matches('\'')
                        .trim_matches('"')
                        .to_string();
                    globs.push(glob);
                } else if !trimmed.is_empty() {
                    break;
                }
            }
        }
        if !globs.is_empty() {
            return Ok(globs);
        }
    }

    // Fall back to package.json workspaces
    let pkg_path = root.join("package.json");
    let content = std::fs::read_to_string(&pkg_path).context("No package.json found")?;
    let root_pkg: RootPackageJson =
        serde_json::from_str(&content).context("Failed to parse root package.json")?;

    match root_pkg.workspaces {
        Some(WorkspacesField::Array(globs)) => Ok(globs),
        Some(WorkspacesField::Object { packages }) => Ok(packages),
        None => anyhow::bail!("No workspaces field found in package.json"),
    }
}

fn expand_globs(root: &Path, globs: &[String]) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();

    for pattern in globs {
        let full_pattern = root.join(pattern).join("package.json");
        let pattern_str = full_pattern.to_str().unwrap_or("");

        match glob::glob(pattern_str) {
            Ok(paths) => {
                for entry in paths.filter_map(|p| p.ok()) {
                    if let Some(parent) = entry.parent() {
                        dirs.push(parent.to_path_buf());
                    }
                }
            }
            Err(_) => continue,
        }
    }

    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_bun_workspace(dir: &std::path::Path) {
        // bun.lock marker
        std::fs::write(dir.join("bun.lock"), "").unwrap();

        // Root package.json with workspaces
        std::fs::write(
            dir.join("package.json"),
            r#"{"name": "root", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();

        // Package A depends on B
        std::fs::create_dir_all(dir.join("packages/pkg-a")).unwrap();
        std::fs::write(
            dir.join("packages/pkg-a/package.json"),
            r#"{"name": "@app/pkg-a", "version": "1.0.0", "dependencies": {"@app/pkg-b": "workspace:*"}}"#,
        )
        .unwrap();

        // Package B (no deps)
        std::fs::create_dir_all(dir.join("packages/pkg-b")).unwrap();
        std::fs::write(
            dir.join("packages/pkg-b/package.json"),
            r#"{"name": "@app/pkg-b", "version": "1.0.0"}"#,
        )
        .unwrap();
    }

    #[test]
    fn test_detect_bun_lock() {
        let dir = tempfile::tempdir().unwrap();
        create_bun_workspace(dir.path());
        assert!(BunResolver.detect(dir.path()));
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
        assert!(BunResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_bunfig() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bunfig.toml"), "").unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "root", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();
        assert!(BunResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_no_bun() {
        let dir = tempfile::tempdir().unwrap();
        // npm workspace without bun marker
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "root", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();
        assert!(!BunResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!BunResolver.detect(dir.path()));
    }

    #[test]
    fn test_resolve_bun_workspace() {
        let dir = tempfile::tempdir().unwrap();
        create_bun_workspace(dir.path());

        let graph = BunResolver.resolve(dir.path()).unwrap();
        assert_eq!(graph.packages.len(), 2);
        assert!(graph
            .packages
            .contains_key(&PackageId("@app/pkg-a".into())));
        assert!(graph
            .packages
            .contains_key(&PackageId("@app/pkg-b".into())));

        // pkg-a depends on pkg-b
        assert!(graph.edges.contains(&(
            PackageId("@app/pkg-a".into()),
            PackageId("@app/pkg-b".into()),
        )));
    }

    #[test]
    fn test_test_command() {
        let cmd = BunResolver.test_command(&PackageId("my-pkg".into()));
        assert_eq!(cmd, vec!["bun", "test", "--filter", "my-pkg"]);
    }

    #[test]
    fn test_dev_dependencies_create_edges() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bun.lock"), "").unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "root", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("packages/lib")).unwrap();
        std::fs::write(
            dir.path().join("packages/lib/package.json"),
            r#"{"name": "lib", "version": "1.0.0"}"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("packages/app")).unwrap();
        std::fs::write(
            dir.path().join("packages/app/package.json"),
            r#"{"name": "app", "version": "1.0.0", "devDependencies": {"lib": "workspace:*"}}"#,
        )
        .unwrap();

        let graph = BunResolver.resolve(dir.path()).unwrap();
        assert!(graph
            .edges
            .contains(&(PackageId("app".into()), PackageId("lib".into()))));
    }
}
