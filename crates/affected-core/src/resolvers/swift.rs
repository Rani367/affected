use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use crate::resolvers::{file_to_package, Resolver};
use crate::types::{Ecosystem, Package, PackageId, ProjectGraph};

/// SwiftResolver detects Swift Package Manager multi-target or multi-package projects.
///
/// Uses regex to parse `Package.swift` manifests for `.target(`, `.executableTarget(`,
/// and `.testTarget(` declarations and their dependency arrays.
pub struct SwiftResolver;

impl Resolver for SwiftResolver {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Swift
    }

    fn detect(&self, root: &Path) -> bool {
        let manifest = root.join("Package.swift");
        if !manifest.exists() {
            return false;
        }

        // Check for multi-package layout: subdirectories containing their own Package.swift
        if has_subdir_packages(root) {
            return true;
        }

        // Check for multi-target single Package.swift
        let content = match std::fs::read_to_string(&manifest) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let target_re =
            Regex::new(r#"\.(target|executableTarget|testTarget)\(\s*name:\s*""#).unwrap();
        let count = target_re.find_iter(&content).count();
        count >= 2
    }

    fn resolve(&self, root: &Path) -> Result<ProjectGraph> {
        // Decide mode: multi-package (subdirs) vs multi-target (single Package.swift)
        if has_subdir_packages(root) {
            self.resolve_multi_package(root)
        } else {
            self.resolve_multi_target(root)
        }
    }

    fn package_for_file(&self, graph: &ProjectGraph, file: &Path) -> Option<PackageId> {
        file_to_package(graph, file)
    }

    fn test_command(&self, package_id: &PackageId) -> Vec<String> {
        vec![
            "swift".into(),
            "test".into(),
            "--filter".into(),
            package_id.0.clone(),
        ]
    }
}

impl SwiftResolver {
    /// Resolve a single Package.swift with multiple targets.
    fn resolve_multi_target(&self, root: &Path) -> Result<ProjectGraph> {
        let manifest = root.join("Package.swift");
        let content = std::fs::read_to_string(&manifest)
            .context("Failed to read Package.swift")?;

        let targets = parse_swift_targets(&content);

        tracing::debug!(
            "Swift: found {} targets: {:?}",
            targets.len(),
            targets.iter().map(|t| &t.name).collect::<Vec<_>>()
        );

        let target_names: std::collections::HashSet<String> =
            targets.iter().map(|t| t.name.clone()).collect();

        let mut packages = HashMap::new();

        for target in &targets {
            let pkg_id = PackageId(target.name.clone());
            let target_dir = root.join("Sources").join(&target.name);
            packages.insert(
                pkg_id.clone(),
                Package {
                    id: pkg_id,
                    name: target.name.clone(),
                    version: None,
                    path: target_dir,
                    manifest_path: manifest.clone(),
                },
            );
        }

        let mut edges = Vec::new();
        for target in &targets {
            let from = PackageId(target.name.clone());
            for dep in &target.dependencies {
                if target_names.contains(dep) && dep != &target.name {
                    edges.push((from.clone(), PackageId(dep.clone())));
                }
            }
        }

        Ok(ProjectGraph {
            packages,
            edges,
            root: root.to_path_buf(),
        })
    }

    /// Resolve multiple Package.swift files in subdirectories.
    fn resolve_multi_package(&self, root: &Path) -> Result<ProjectGraph> {
        let mut packages = HashMap::new();

        let entries = std::fs::read_dir(root)
            .context("Failed to read project root directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest = path.join("Package.swift");
            if !manifest.exists() {
                continue;
            }

            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let pkg_id = PackageId(dir_name.clone());
            packages.insert(
                pkg_id.clone(),
                Package {
                    id: pkg_id,
                    name: dir_name,
                    version: None,
                    path: path.clone(),
                    manifest_path: manifest,
                },
            );
        }

        tracing::debug!(
            "Swift: found {} sub-packages: {:?}",
            packages.len(),
            packages.keys().collect::<Vec<_>>()
        );

        // Build dependency edges by scanning each Package.swift for .package(path: "...") refs
        let package_names: std::collections::HashSet<String> =
            packages.keys().map(|k| k.0.clone()).collect();

        let path_dep_re = Regex::new(r#"\.package\(\s*path:\s*"([^"]+)""#).unwrap();
        let mut edges = Vec::new();
        for (pkg_id, pkg) in &packages {
            let content = std::fs::read_to_string(&pkg.manifest_path)
                .with_context(|| format!("Failed to read {}", pkg.manifest_path.display()))?;

            for cap in path_dep_re.captures_iter(&content) {
                if let Some(dep_path) = cap.get(1) {
                    // Extract directory name from path (e.g., "../Core" -> "Core")
                    let dep_name = Path::new(dep_path.as_str())
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    if package_names.contains(&dep_name) && dep_name != pkg_id.0 {
                        edges.push((pkg_id.clone(), PackageId(dep_name)));
                    }
                }
            }
        }

        Ok(ProjectGraph {
            packages,
            edges,
            root: root.to_path_buf(),
        })
    }
}

/// Check whether subdirectories of `root` contain their own `Package.swift`.
fn has_subdir_packages(root: &Path) -> bool {
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return false,
    };

    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("Package.swift").exists() {
            count += 1;
            if count >= 2 {
                return true;
            }
        }
    }

    false
}

/// A parsed Swift target with its name and local dependencies.
#[derive(Debug)]
struct SwiftTarget {
    name: String,
    dependencies: Vec<String>,
}

/// Parse target declarations from a `Package.swift` file.
///
/// Extracts `.target(name: "X", ...)`, `.executableTarget(name: "X", ...)`,
/// and `.testTarget(name: "X", ...)` blocks, along with the dependency names
/// found in each target's `dependencies:` array.
fn parse_swift_targets(content: &str) -> Vec<SwiftTarget> {
    let mut targets = Vec::new();

    // First pass: find all target declarations and their positions.
    let target_re =
        Regex::new(r#"\.(target|executableTarget|testTarget)\(\s*name:\s*"([^"]+)""#).unwrap();

    let target_matches: Vec<(usize, String)> = target_re
        .captures_iter(content)
        .filter_map(|cap| {
            let pos = cap.get(0)?.start();
            let name = cap.get(2)?.as_str().to_string();
            Some((pos, name))
        })
        .collect();

    // Second pass: for each target, extract the block up to a reasonable boundary
    // and parse dependencies from it.
    for (i, (pos, name)) in target_matches.iter().enumerate() {
        // Determine the end of this target's text: either the start of the next target
        // or end of file.
        let end = if i + 1 < target_matches.len() {
            target_matches[i + 1].0
        } else {
            content.len()
        };

        let block = &content[*pos..end];
        let deps = parse_target_dependencies(block);

        targets.push(SwiftTarget {
            name: name.clone(),
            dependencies: deps,
        });
    }

    targets
}

/// Parse dependency names from a target block.
///
/// Looks for the `dependencies:` array and extracts:
/// - Bare string dependencies: `"Foo"`
/// - `.product(name: "Foo", ...)` references
fn parse_target_dependencies(block: &str) -> Vec<String> {
    let mut deps = Vec::new();

    // Find the dependencies: [...] array within this block.
    let deps_start = match block.find("dependencies:") {
        Some(pos) => pos,
        None => return deps,
    };

    let after_deps = &block[deps_start..];

    // Find the opening bracket
    let bracket_start = match after_deps.find('[') {
        Some(pos) => pos,
        None => return deps,
    };

    // Find the matching closing bracket (handle nested brackets)
    let bracket_content = &after_deps[bracket_start..];
    let mut depth = 0;
    let mut end_pos = bracket_content.len();
    for (i, ch) in bracket_content.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end_pos = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    let deps_array = &bracket_content[..end_pos];

    // Match .product(name: "Foo" references
    let product_re = Regex::new(r#"\.product\(\s*name:\s*"([^"]+)""#).unwrap();
    for cap in product_re.captures_iter(deps_array) {
        if let Some(name) = cap.get(1) {
            let dep = name.as_str().to_string();
            if !deps.contains(&dep) {
                deps.push(dep);
            }
        }
    }

    // Match bare string dependencies: standalone "Foo" not inside .product(...)
    // We strip out .product(...) blocks first, then find remaining quoted strings.
    let stripped = product_re.replace_all(deps_array, "");
    // Also strip .target(name: ... and .byName(name: ... references first
    let target_dep_re = Regex::new(r#"\.(target|byName)\(\s*name:\s*"([^"]+)""#).unwrap();
    for cap in target_dep_re.captures_iter(deps_array) {
        if let Some(name) = cap.get(2) {
            let dep = name.as_str().to_string();
            if !deps.contains(&dep) {
                deps.push(dep);
            }
        }
    }
    let stripped = target_dep_re.replace_all(&stripped, "");

    let bare_re = Regex::new(r#""([^"]+)""#).unwrap();
    for cap in bare_re.captures_iter(&stripped) {
        if let Some(name) = cap.get(1) {
            let dep = name.as_str().to_string();
            if !deps.contains(&dep) {
                deps.push(dep);
            }
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PACKAGE_SWIFT: &str = r#"
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "MyPackage",
    targets: [
        .target(name: "Core", dependencies: []),
        .target(name: "Networking", dependencies: ["Core"]),
        .executableTarget(name: "CLI", dependencies: ["Core", "Networking"]),
        .testTarget(name: "CoreTests", dependencies: ["Core"]),
    ]
)
"#;

    #[test]
    fn test_detect_package_swift() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Package.swift"), SAMPLE_PACKAGE_SWIFT).unwrap();
        assert!(SwiftResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_single_target_no_detect() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
let package = Package(
    name: "SingleLib",
    targets: [
        .target(name: "SingleLib", dependencies: []),
    ]
)
"#;
        std::fs::write(dir.path().join("Package.swift"), content).unwrap();
        assert!(!SwiftResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_no_swift() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!SwiftResolver.detect(dir.path()));
    }

    #[test]
    fn test_parse_swift_targets() {
        let targets = parse_swift_targets(SAMPLE_PACKAGE_SWIFT);
        let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["Core", "Networking", "CLI", "CoreTests"]);
    }

    #[test]
    fn test_resolve_swift_package() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Package.swift"), SAMPLE_PACKAGE_SWIFT).unwrap();

        // Create target source directories (swift convention: Sources/<Target>/)
        for name in &["Core", "Networking", "CLI", "CoreTests"] {
            std::fs::create_dir_all(dir.path().join("Sources").join(name)).unwrap();
        }

        let graph = SwiftResolver.resolve(dir.path()).unwrap();

        assert_eq!(graph.packages.len(), 4);
        assert!(graph.packages.contains_key(&PackageId("Core".into())));
        assert!(graph.packages.contains_key(&PackageId("Networking".into())));
        assert!(graph.packages.contains_key(&PackageId("CLI".into())));
        assert!(graph.packages.contains_key(&PackageId("CoreTests".into())));

        // Networking depends on Core
        assert!(graph
            .edges
            .contains(&(PackageId("Networking".into()), PackageId("Core".into()))));

        // CLI depends on Core and Networking
        assert!(graph
            .edges
            .contains(&(PackageId("CLI".into()), PackageId("Core".into()))));
        assert!(graph
            .edges
            .contains(&(PackageId("CLI".into()), PackageId("Networking".into()))));

        // CoreTests depends on Core
        assert!(graph
            .edges
            .contains(&(PackageId("CoreTests".into()), PackageId("Core".into()))));

        // Core has no internal dependencies
        let core_deps: Vec<_> = graph
            .edges
            .iter()
            .filter(|(from, _)| from.0 == "Core")
            .collect();
        assert!(core_deps.is_empty());
    }

    #[test]
    fn test_test_command() {
        let cmd = SwiftResolver.test_command(&PackageId("Core".into()));
        assert_eq!(cmd, vec!["swift", "test", "--filter", "Core"]);
    }

    #[test]
    fn test_detect_multi_package() {
        let dir = tempfile::tempdir().unwrap();

        // Root Package.swift with only one target (would not qualify on its own)
        std::fs::write(
            dir.path().join("Package.swift"),
            r#"let package = Package(name: "Root", targets: [.target(name: "Root", dependencies: [])])"#,
        )
        .unwrap();

        // Two subdirectory packages
        std::fs::create_dir_all(dir.path().join("CoreLib")).unwrap();
        std::fs::write(
            dir.path().join("CoreLib/Package.swift"),
            r#"let package = Package(name: "CoreLib")"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("NetLib")).unwrap();
        std::fs::write(
            dir.path().join("NetLib/Package.swift"),
            r#"let package = Package(name: "NetLib")"#,
        )
        .unwrap();

        assert!(SwiftResolver.detect(dir.path()));
    }

    #[test]
    fn test_resolve_multi_package() {
        let dir = tempfile::tempdir().unwrap();

        std::fs::create_dir_all(dir.path().join("CoreLib")).unwrap();
        std::fs::write(
            dir.path().join("CoreLib/Package.swift"),
            r#"let package = Package(name: "CoreLib", targets: [.target(name: "CoreLib")])"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("NetLib")).unwrap();
        std::fs::write(
            dir.path().join("NetLib/Package.swift"),
            r#"
let package = Package(
    name: "NetLib",
    dependencies: [.package(path: "../CoreLib")],
    targets: [.target(name: "NetLib")]
)
"#,
        )
        .unwrap();

        let graph = SwiftResolver.resolve_multi_package(dir.path()).unwrap();

        assert_eq!(graph.packages.len(), 2);
        assert!(graph.packages.contains_key(&PackageId("CoreLib".into())));
        assert!(graph.packages.contains_key(&PackageId("NetLib".into())));

        // NetLib depends on CoreLib
        assert!(graph
            .edges
            .contains(&(PackageId("NetLib".into()), PackageId("CoreLib".into()))));
    }

    #[test]
    fn test_parse_target_with_product_dependency() {
        let content = r#"
let package = Package(
    name: "MyApp",
    targets: [
        .target(name: "App", dependencies: [
            .product(name: "Logging", package: "swift-log"),
            "Core",
        ]),
        .target(name: "Core", dependencies: []),
    ]
)
"#;
        let targets = parse_swift_targets(content);
        assert_eq!(targets.len(), 2);

        let app = &targets[0];
        assert_eq!(app.name, "App");
        assert!(app.dependencies.contains(&"Logging".to_string()));
        assert!(app.dependencies.contains(&"Core".to_string()));
    }
}
