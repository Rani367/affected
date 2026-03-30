use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::resolvers::{file_to_package, Resolver};
use crate::types::{Ecosystem, Package, PackageId, ProjectGraph};

pub struct DartResolver;

/// Which monorepo mode we detected.
enum DartMode {
    /// Dart 3.6+ native workspaces — root `pubspec.yaml` has a `workspace:` section.
    Workspace,
    /// Melos monorepo tool — `melos.yaml` at root.
    Melos,
    /// Generic — 2+ `pubspec.yaml` files in immediate subdirectories.
    Generic,
}

/// Extract the `name:` field from a `pubspec.yaml` file's content.
fn parse_pubspec_name(content: &str) -> Option<String> {
    for line in content.lines() {
        // Only match top-level `name:` (no leading whitespace)
        if line.starts_with("name:") {
            let value = line.trim_start_matches("name:").trim();
            // Strip optional surrounding quotes
            let value = value.trim_matches('\'').trim_matches('"');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Extract dependency names from both `dependencies:` and `dev_dependencies:` sections
/// of a `pubspec.yaml` file's content.
///
/// We parse line-by-line, tracking when we enter a dep section (indentation level 0
/// for the section header, indented entries underneath). Each top-level key inside
/// the section is treated as a dependency name.
fn parse_pubspec_deps(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_dep_section = false;

    for line in content.lines() {
        // Skip blank lines — they don't end sections in YAML
        if line.trim().is_empty() {
            continue;
        }

        // A top-level key (no leading whitespace)
        let is_top_level = !line.starts_with(' ') && !line.starts_with('\t');

        if is_top_level {
            let trimmed = line.trim();
            if trimmed == "dependencies:" || trimmed == "dev_dependencies:" {
                in_dep_section = true;
                continue;
            } else {
                // Any other top-level key ends the dep section
                in_dep_section = false;
                continue;
            }
        }

        if in_dep_section {
            let stripped = line.trim();

            // Determine indent: count leading spaces
            let indent = line.len() - line.trim_start().len();

            // We treat indent == 2..=4 as a direct child entry (typical pubspec uses 2).
            // Deeper indentation belongs to the previous entry's value block.
            if (2..=4).contains(&indent) && stripped.contains(':') {
                let dep_name = stripped.split(':').next().unwrap_or("").trim();
                if !dep_name.is_empty() && !dep_name.starts_with('#') {
                    deps.push(dep_name.to_string());
                }
            }
        }
    }

    deps
}

impl Resolver for DartResolver {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Dart
    }

    fn detect(&self, root: &Path) -> bool {
        // Mode 1: Dart 3.6+ native workspace
        let root_pubspec = root.join("pubspec.yaml");
        if root_pubspec.exists() {
            if let Ok(content) = std::fs::read_to_string(&root_pubspec) {
                for line in content.lines() {
                    if !line.starts_with(' ') && !line.starts_with('\t') {
                        let trimmed = line.trim();
                        if trimmed == "workspace:" || trimmed.starts_with("workspace:") {
                            return true;
                        }
                    }
                }
            }
        }

        // Mode 2: Melos
        if root.join("melos.yaml").exists() {
            return true;
        }

        // Mode 3: Generic — 2+ pubspec.yaml in immediate subdirs
        if let Ok(entries) = std::fs::read_dir(root) {
            let count = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter(|e| e.path().join("pubspec.yaml").exists())
                .count();
            if count >= 2 {
                return true;
            }
        }

        false
    }

    fn resolve(&self, root: &Path) -> Result<ProjectGraph> {
        let mode = self.detect_mode(root)?;
        let pkg_dirs = match mode {
            DartMode::Workspace => self.resolve_workspace(root)?,
            DartMode::Melos => self.resolve_melos(root)?,
            DartMode::Generic => self.resolve_generic(root)?,
        };

        // Parse all workspace packages
        let mut packages = HashMap::new();
        let mut name_to_id = HashMap::new();

        for dir in &pkg_dirs {
            let pubspec_path = dir.join("pubspec.yaml");
            if !pubspec_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&pubspec_path)
                .with_context(|| format!("Failed to read {}", pubspec_path.display()))?;

            let name = match parse_pubspec_name(&content) {
                Some(n) => n,
                None => continue,
            };

            let pkg_id = PackageId(name.clone());
            name_to_id.insert(name.clone(), pkg_id.clone());
            packages.insert(
                pkg_id.clone(),
                Package {
                    id: pkg_id,
                    name: name.clone(),
                    version: None,
                    path: dir.clone(),
                    manifest_path: pubspec_path,
                },
            );
        }

        // Build dependency edges
        let mut edges = Vec::new();
        let workspace_names: std::collections::HashSet<&str> =
            name_to_id.keys().map(|s| s.as_str()).collect();

        for dir in &pkg_dirs {
            let pubspec_path = dir.join("pubspec.yaml");
            if !pubspec_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&pubspec_path)?;

            let from_name = match parse_pubspec_name(&content) {
                Some(n) => n,
                None => continue,
            };

            let all_deps = parse_pubspec_deps(&content);

            for dep_name in all_deps {
                if workspace_names.contains(dep_name.as_str()) {
                    edges.push((
                        PackageId(from_name.clone()),
                        PackageId(dep_name),
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
            "dart".into(),
            "test".into(),
            "-C".into(),
            package_id.0.clone(),
        ]
    }
}

impl DartResolver {
    /// Determine which monorepo mode the project uses.
    fn detect_mode(&self, root: &Path) -> Result<DartMode> {
        // Priority 1: Dart 3.6+ native workspace
        let root_pubspec = root.join("pubspec.yaml");
        if root_pubspec.exists() {
            if let Ok(content) = std::fs::read_to_string(&root_pubspec) {
                for line in content.lines() {
                    if !line.starts_with(' ') && !line.starts_with('\t') {
                        let trimmed = line.trim();
                        if trimmed == "workspace:" || trimmed.starts_with("workspace:") {
                            return Ok(DartMode::Workspace);
                        }
                    }
                }
            }
        }

        // Priority 2: Melos
        if root.join("melos.yaml").exists() {
            return Ok(DartMode::Melos);
        }

        // Priority 3: Generic
        Ok(DartMode::Generic)
    }

    /// Parse the root `pubspec.yaml` for a `workspace:` section containing a list of paths.
    ///
    /// ```yaml
    /// workspace:
    ///   - packages/core
    ///   - packages/api
    /// ```
    fn resolve_workspace(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let pubspec_path = root.join("pubspec.yaml");
        let content = std::fs::read_to_string(&pubspec_path)
            .context("Failed to read root pubspec.yaml")?;

        let mut dirs = Vec::new();
        let mut in_workspace = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Detect top-level `workspace:` key
            if !line.starts_with(' ') && !line.starts_with('\t') {
                if trimmed == "workspace:" {
                    in_workspace = true;
                    continue;
                } else if in_workspace {
                    // Another top-level key ends the workspace section
                    break;
                }
                continue;
            }

            if in_workspace {
                if trimmed.starts_with("- ") {
                    let path = trimmed
                        .trim_start_matches("- ")
                        .trim_matches('\'')
                        .trim_matches('"')
                        .to_string();
                    let abs = root.join(&path);
                    if abs.is_dir() {
                        dirs.push(abs);
                    }
                } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    break;
                }
            }
        }

        Ok(dirs)
    }

    /// Parse `melos.yaml` for the `packages:` field containing glob patterns,
    /// then expand them to find directories containing `pubspec.yaml`.
    ///
    /// ```yaml
    /// packages:
    ///   - packages/*
    /// ```
    fn resolve_melos(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let melos_path = root.join("melos.yaml");
        let content = std::fs::read_to_string(&melos_path)
            .context("Failed to read melos.yaml")?;

        let mut globs = Vec::new();
        let mut in_packages = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if !line.starts_with(' ') && !line.starts_with('\t') {
                if trimmed == "packages:" {
                    in_packages = true;
                    continue;
                } else if in_packages {
                    break;
                }
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
                } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    break;
                }
            }
        }

        self.expand_globs(root, &globs)
    }

    /// Scan immediate subdirectories for `pubspec.yaml` files.
    fn resolve_generic(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();

        let entries = std::fs::read_dir(root)
            .with_context(|| format!("Failed to read directory {}", root.display()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() && path.join("pubspec.yaml").exists() {
                dirs.push(path);
            }
        }

        // Sort for deterministic ordering
        dirs.sort();
        Ok(dirs)
    }

    /// Expand glob patterns to find directories containing `pubspec.yaml`.
    fn expand_globs(&self, root: &Path, globs: &[String]) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();

        for pattern in globs {
            let full_pattern = root.join(pattern).join("pubspec.yaml");
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a Dart 3.6+ native workspace layout.
    fn create_dart_workspace(dir: &Path) {
        // Root pubspec.yaml with workspace members
        std::fs::write(
            dir.join("pubspec.yaml"),
            "name: my_workspace\n\
             workspace:\n  - packages/core\n  - packages/api\n",
        )
        .unwrap();

        // packages/core
        std::fs::create_dir_all(dir.join("packages/core")).unwrap();
        std::fs::write(
            dir.join("packages/core/pubspec.yaml"),
            "name: core\n\n\
             dependencies:\n  flutter:\n    sdk: flutter\n",
        )
        .unwrap();

        // packages/api depends on core
        std::fs::create_dir_all(dir.join("packages/api")).unwrap();
        std::fs::write(
            dir.join("packages/api/pubspec.yaml"),
            "name: api\n\n\
             dependencies:\n  core:\n    path: ../core\n\n\
             dev_dependencies:\n  test: ^1.0.0\n",
        )
        .unwrap();
    }

    /// Create a Melos monorepo layout.
    fn create_melos_project(dir: &Path) {
        std::fs::write(
            dir.join("melos.yaml"),
            "name: my_project\npackages:\n  - packages/*\n",
        )
        .unwrap();

        std::fs::write(
            dir.join("pubspec.yaml"),
            "name: my_project\n",
        )
        .unwrap();

        // packages/alpha
        std::fs::create_dir_all(dir.join("packages/alpha")).unwrap();
        std::fs::write(
            dir.join("packages/alpha/pubspec.yaml"),
            "name: alpha\n\n\
             dependencies:\n  beta:\n    path: ../beta\n",
        )
        .unwrap();

        // packages/beta
        std::fs::create_dir_all(dir.join("packages/beta")).unwrap();
        std::fs::write(
            dir.join("packages/beta/pubspec.yaml"),
            "name: beta\n",
        )
        .unwrap();
    }

    #[test]
    fn test_detect_dart_workspace() {
        let dir = tempfile::tempdir().unwrap();
        create_dart_workspace(dir.path());
        assert!(DartResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_melos() {
        let dir = tempfile::tempdir().unwrap();
        create_melos_project(dir.path());
        assert!(DartResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_generic_multiple_pubspecs() {
        let dir = tempfile::tempdir().unwrap();

        // Two subdirs with pubspec.yaml, no workspace key, no melos.yaml
        std::fs::create_dir_all(dir.path().join("app_a")).unwrap();
        std::fs::write(
            dir.path().join("app_a/pubspec.yaml"),
            "name: app_a\n",
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("app_b")).unwrap();
        std::fs::write(
            dir.path().join("app_b/pubspec.yaml"),
            "name: app_b\n",
        )
        .unwrap();

        assert!(DartResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_no_dart() {
        let dir = tempfile::tempdir().unwrap();
        // Empty directory — nothing to detect
        assert!(!DartResolver.detect(dir.path()));

        // Single pubspec.yaml without workspace key is not enough
        let dir2 = tempfile::tempdir().unwrap();
        std::fs::write(
            dir2.path().join("pubspec.yaml"),
            "name: solo_app\n",
        )
        .unwrap();
        assert!(!DartResolver.detect(dir2.path()));

        // Single subdir with pubspec.yaml is not enough for generic mode
        let dir3 = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir3.path().join("only_one")).unwrap();
        std::fs::write(
            dir3.path().join("only_one/pubspec.yaml"),
            "name: only_one\n",
        )
        .unwrap();
        assert!(!DartResolver.detect(dir3.path()));
    }

    #[test]
    fn test_resolve_dart_workspace() {
        let dir = tempfile::tempdir().unwrap();
        create_dart_workspace(dir.path());

        let graph = DartResolver.resolve(dir.path()).unwrap();

        // Should discover core and api
        assert_eq!(graph.packages.len(), 2);
        assert!(graph.packages.contains_key(&PackageId("core".into())));
        assert!(graph.packages.contains_key(&PackageId("api".into())));

        // api depends on core
        assert!(graph.edges.contains(&(
            PackageId("api".into()),
            PackageId("core".into()),
        )));

        // Verify paths
        let core_pkg = &graph.packages[&PackageId("core".into())];
        assert!(core_pkg.path.ends_with("packages/core"));
        assert!(core_pkg
            .manifest_path
            .ends_with("packages/core/pubspec.yaml"));
    }

    #[test]
    fn test_resolve_melos_project() {
        let dir = tempfile::tempdir().unwrap();
        create_melos_project(dir.path());

        let graph = DartResolver.resolve(dir.path()).unwrap();

        // Should discover alpha and beta
        assert_eq!(graph.packages.len(), 2);
        assert!(graph.packages.contains_key(&PackageId("alpha".into())));
        assert!(graph.packages.contains_key(&PackageId("beta".into())));

        // alpha depends on beta
        assert!(graph.edges.contains(&(
            PackageId("alpha".into()),
            PackageId("beta".into()),
        )));
    }

    #[test]
    fn test_test_command() {
        let cmd = DartResolver.test_command(&PackageId("my_package".into()));
        assert_eq!(cmd, vec!["dart", "test", "-C", "my_package"]);
    }

    #[test]
    fn test_parse_pubspec_name() {
        assert_eq!(
            parse_pubspec_name("name: my_app\nversion: 1.0.0\n"),
            Some("my_app".into()),
        );
        assert_eq!(
            parse_pubspec_name("name: 'quoted_name'\n"),
            Some("quoted_name".into()),
        );
        assert_eq!(parse_pubspec_name("version: 1.0.0\n"), None);
    }

    #[test]
    fn test_parse_pubspec_deps() {
        let content = "\
name: my_app

dependencies:
  core:
    path: ../core
  http: ^0.13.0

dev_dependencies:
  test_utils:
    path: ../test_utils
  lints: ^2.0.0

flutter:
  uses-material-design: true
";
        let deps = parse_pubspec_deps(content);
        assert!(deps.contains(&"core".to_string()));
        assert!(deps.contains(&"http".to_string()));
        assert!(deps.contains(&"test_utils".to_string()));
        assert!(deps.contains(&"lints".to_string()));
        // `flutter` is a top-level key, not a dep
        assert!(!deps.contains(&"flutter".to_string()));
    }

    #[test]
    fn test_resolve_generic_mode() {
        let dir = tempfile::tempdir().unwrap();

        // Two subdirs with pubspec.yaml, one depends on the other
        std::fs::create_dir_all(dir.path().join("pkg_a")).unwrap();
        std::fs::write(
            dir.path().join("pkg_a/pubspec.yaml"),
            "name: pkg_a\n\n\
             dependencies:\n  pkg_b:\n    path: ../pkg_b\n",
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("pkg_b")).unwrap();
        std::fs::write(
            dir.path().join("pkg_b/pubspec.yaml"),
            "name: pkg_b\n",
        )
        .unwrap();

        let graph = DartResolver.resolve(dir.path()).unwrap();
        assert_eq!(graph.packages.len(), 2);
        assert!(graph.edges.contains(&(
            PackageId("pkg_a".into()),
            PackageId("pkg_b".into()),
        )));
    }

    #[test]
    fn test_dev_dependencies_create_edges() {
        let dir = tempfile::tempdir().unwrap();

        std::fs::write(
            dir.path().join("pubspec.yaml"),
            "name: root_ws\nworkspace:\n  - packages/lib\n  - packages/app\n",
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("packages/lib")).unwrap();
        std::fs::write(
            dir.path().join("packages/lib/pubspec.yaml"),
            "name: lib\n",
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("packages/app")).unwrap();
        std::fs::write(
            dir.path().join("packages/app/pubspec.yaml"),
            "name: app\n\n\
             dev_dependencies:\n  lib:\n    path: ../lib\n",
        )
        .unwrap();

        let graph = DartResolver.resolve(dir.path()).unwrap();
        assert!(graph.edges.contains(&(
            PackageId("app".into()),
            PackageId("lib".into()),
        )));
    }
}
