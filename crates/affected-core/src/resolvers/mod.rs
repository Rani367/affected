use anyhow::Result;
use std::path::Path;

use crate::types::{Ecosystem, PackageId, ProjectGraph};

pub mod bun;
pub mod cargo;
pub mod dart;
pub mod dotnet;
pub mod elixir;
pub mod go;
pub mod gradle;
pub mod maven;
pub mod npm;
pub mod python;
pub mod sbt;
pub mod swift;
pub mod yarn;

/// Trait implemented by each ecosystem resolver.
pub trait Resolver {
    /// Which ecosystem this resolver handles.
    fn ecosystem(&self) -> Ecosystem;

    /// Can this resolver handle the project at the given root path?
    fn detect(&self, root: &Path) -> bool;

    /// Build the full project graph: packages + dependency edges.
    fn resolve(&self, root: &Path) -> Result<ProjectGraph>;

    /// Given a file path (relative to project root), return which package owns it.
    fn package_for_file(&self, graph: &ProjectGraph, file: &Path) -> Option<PackageId>;

    /// Return the shell command to run tests for a given package.
    fn test_command(&self, package_id: &PackageId) -> Vec<String>;
}

/// Return all available resolvers.
pub fn all_resolvers() -> Vec<Box<dyn Resolver>> {
    vec![
        Box::new(cargo::CargoResolver),
        Box::new(yarn::YarnResolver), // Yarn before Bun/Npm: .yarnrc.yml takes priority
        Box::new(bun::BunResolver),   // Bun before Npm: bun.lock takes priority
        Box::new(npm::NpmResolver),
        Box::new(go::GoResolver),
        Box::new(python::PythonResolver),
        Box::new(maven::MavenResolver),
        Box::new(gradle::GradleResolver),
        Box::new(dotnet::DotnetResolver),
        Box::new(swift::SwiftResolver),
        Box::new(dart::DartResolver),
        Box::new(elixir::ElixirResolver),
        Box::new(sbt::SbtResolver),
    ]
}

/// Auto-select the first matching resolver for a project.
pub fn detect_resolver(root: &Path) -> Result<Box<dyn Resolver>> {
    for resolver in all_resolvers() {
        if resolver.detect(root) {
            return Ok(resolver);
        }
    }
    anyhow::bail!("No supported project type detected at {}", root.display())
}

/// Map a file to its owning package using longest-prefix directory matching.
pub fn file_to_package(graph: &ProjectGraph, file: &Path) -> Option<PackageId> {
    let mut best: Option<(&PackageId, usize)> = None;

    for (id, pkg) in &graph.packages {
        // Get package path relative to project root
        let pkg_rel = pkg.path.strip_prefix(&graph.root).unwrap_or(&pkg.path);

        if file.starts_with(pkg_rel) {
            let depth = pkg_rel.components().count();
            if best.is_none() || depth > best.unwrap().1 {
                best = Some((id, depth));
            }
        }
    }

    best.map(|(id, _)| id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Package;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_project_graph(pkgs: &[(&str, &str)]) -> ProjectGraph {
        let root = PathBuf::from("/project");
        let mut packages = HashMap::new();
        for (name, rel_path) in pkgs {
            let id = PackageId(name.to_string());
            packages.insert(
                id.clone(),
                Package {
                    id: id.clone(),
                    name: name.to_string(),
                    version: None,
                    path: root.join(rel_path),
                    manifest_path: root.join(rel_path).join("Cargo.toml"),
                },
            );
        }
        ProjectGraph {
            packages,
            edges: vec![],
            root,
        }
    }

    #[test]
    fn test_file_to_package_basic() {
        let pg = make_project_graph(&[("core", "crates/core"), ("cli", "crates/cli")]);
        let result = file_to_package(&pg, &PathBuf::from("crates/core/src/lib.rs"));
        assert_eq!(result, Some(PackageId("core".into())));
    }

    #[test]
    fn test_file_to_package_no_match() {
        let pg = make_project_graph(&[("core", "crates/core")]);
        let result = file_to_package(&pg, &PathBuf::from("scripts/build.sh"));
        assert!(result.is_none());
    }

    #[test]
    fn test_file_to_package_longest_prefix() {
        // Nested packages: "crates/foo" and "crates/foo/bar"
        let pg = make_project_graph(&[("foo", "crates/foo"), ("foo-bar", "crates/foo/bar")]);

        // File in nested package should match the deeper one
        let result = file_to_package(&pg, &PathBuf::from("crates/foo/bar/src/lib.rs"));
        assert_eq!(result, Some(PackageId("foo-bar".into())));

        // File in parent package should match the shallower one
        let result = file_to_package(&pg, &PathBuf::from("crates/foo/src/lib.rs"));
        assert_eq!(result, Some(PackageId("foo".into())));
    }

    #[test]
    fn test_file_to_package_root_level_file() {
        let pg = make_project_graph(&[("core", "crates/core")]);
        let result = file_to_package(&pg, &PathBuf::from("README.md"));
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_resolver_cargo() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();

        let resolver = detect_resolver(dir.path()).unwrap();
        assert_eq!(resolver.ecosystem(), Ecosystem::Cargo);
    }

    #[test]
    fn test_detect_resolver_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(detect_resolver(dir.path()).is_err());
    }

    #[test]
    fn test_all_resolvers_count() {
        assert_eq!(all_resolvers().len(), 13);
    }
}
