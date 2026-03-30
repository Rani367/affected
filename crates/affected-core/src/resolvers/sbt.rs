use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use crate::resolvers::{file_to_package, Resolver};
use crate::types::{Ecosystem, Package, PackageId, ProjectGraph};

/// SbtResolver detects Scala sbt multi-project builds via `build.sbt`.
///
/// Uses regex to parse `lazy val` project declarations and `.dependsOn()` dependency references.
pub struct SbtResolver;

impl Resolver for SbtResolver {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Sbt
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("build.sbt").exists()
    }

    fn resolve(&self, root: &Path) -> Result<ProjectGraph> {
        let build_sbt_path = root.join("build.sbt");
        let content = std::fs::read_to_string(&build_sbt_path)
            .context("Failed to read build.sbt")?;

        let projects = parse_sbt_projects(&content);
        let dependencies = parse_sbt_dependencies(&content);

        tracing::debug!(
            "Sbt: found {} projects: {:?}",
            projects.len(),
            projects
        );

        // Build a mapping from variable name to PackageId
        let var_to_id: HashMap<String, PackageId> = projects
            .iter()
            .map(|(var_name, _)| (var_name.clone(), PackageId(var_name.clone())))
            .collect();

        let mut packages = HashMap::new();

        for (var_name, dir_path) in &projects {
            let module_dir = root.join(dir_path);

            if !module_dir.exists() {
                tracing::debug!(
                    "Sbt: project '{}' directory '{}' does not exist, skipping",
                    var_name,
                    dir_path
                );
                continue;
            }

            let pkg_id = PackageId(var_name.clone());
            let manifest_path = build_sbt_path.clone();

            packages.insert(
                pkg_id.clone(),
                Package {
                    id: pkg_id,
                    name: var_name.clone(),
                    version: None,
                    path: module_dir,
                    manifest_path,
                },
            );
        }

        // Build dependency edges from .dependsOn() references
        let mut edges = Vec::new();

        for (var_name, deps) in &dependencies {
            if let Some(from_id) = var_to_id.get(var_name) {
                // Only add edges for packages we actually resolved
                if !packages.contains_key(from_id) {
                    continue;
                }

                for dep_name in deps {
                    if let Some(to_id) = var_to_id.get(dep_name) {
                        if packages.contains_key(to_id) && to_id != from_id {
                            edges.push((from_id.clone(), to_id.clone()));
                        }
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

    fn package_for_file(&self, graph: &ProjectGraph, file: &Path) -> Option<PackageId> {
        file_to_package(graph, file)
    }

    fn test_command(&self, package_id: &PackageId) -> Vec<String> {
        vec!["sbt".into(), format!("{}/test", package_id.0)]
    }
}

/// Parse `lazy val` project declarations from a `build.sbt` file.
///
/// Handles two forms:
/// - `lazy val core = (project in file("core"))` -- explicit directory
/// - `lazy val core = project` -- directory defaults to the variable name
///
/// Returns a vec of `(variable_name, directory_path)` tuples.
fn parse_sbt_projects(content: &str) -> Vec<(String, String)> {
    let mut projects = Vec::new();
    let mut matched_vars: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Pattern for `lazy val foo = (project in file("bar"))`
    let re_with_file =
        Regex::new(r#"lazy\s+val\s+(\w+)\s*=\s*\(?\s*project\s+in\s+file\("([^"]+)"\)"#)
            .unwrap();

    // Pattern for bare `lazy val foo = project` (end of line or followed by newline + dot)
    let re_bare_eol = Regex::new(r#"lazy\s+val\s+(\w+)\s*=\s*\(?\s*project\s*$"#).unwrap();
    let re_bare_chain =
        Regex::new(r#"lazy\s+val\s+(\w+)\s*=\s*\(?\s*project\s*\n\s*\."#).unwrap();

    // First, find all projects with explicit file("...") declarations
    for cap in re_with_file.captures_iter(content) {
        let var_name = cap[1].to_string();
        let dir_path = cap[2].to_string();
        matched_vars.insert(var_name.clone());
        projects.push((var_name, dir_path));
    }

    // Find bare `lazy val foo = project` at end of line
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(cap) = re_bare_eol.captures(trimmed) {
            let var_name = cap[1].to_string();
            if !matched_vars.contains(&var_name) {
                matched_vars.insert(var_name.clone());
                projects.push((var_name.clone(), var_name));
            }
        }
    }

    // Also handle bare project with chained calls (multiline)
    for cap in re_bare_chain.captures_iter(content) {
        let var_name = cap[1].to_string();
        if !matched_vars.contains(&var_name) {
            matched_vars.insert(var_name.clone());
            projects.push((var_name.clone(), var_name));
        }
    }

    projects
}

/// Parse `.dependsOn()` dependency declarations from a `build.sbt` file.
///
/// Handles:
/// - `.dependsOn(a)` -- single dependency
/// - `.dependsOn(a, b)` -- multiple comma-separated dependencies
/// - `.dependsOn(a).dependsOn(b)` -- chained calls
///
/// Returns a map of variable_name -> vec of dependency variable names.
fn parse_sbt_dependencies(content: &str) -> HashMap<String, Vec<String>> {
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();

    let lazy_val_re = Regex::new(r#"lazy\s+val\s+(\w+)\s*="#).unwrap();
    let depends_on_re = Regex::new(r#"\.dependsOn\(([^)]+)\)"#).unwrap();

    // Find the byte offset of each `lazy val` declaration to split into blocks.
    let mut block_starts: Vec<(String, usize)> = Vec::new();
    for cap in lazy_val_re.captures_iter(content) {
        let var_name = cap[1].to_string();
        let start = cap.get(0).unwrap().start();
        block_starts.push((var_name, start));
    }

    // Process each block: from this lazy val to the next (or end of string)
    for i in 0..block_starts.len() {
        let (ref var_name, start) = block_starts[i];
        let end = if i + 1 < block_starts.len() {
            block_starts[i + 1].1
        } else {
            content.len()
        };
        let block = &content[start..end];

        let mut var_deps = Vec::new();

        for dep_cap in depends_on_re.captures_iter(block) {
            let dep_list = &dep_cap[1];
            for dep in dep_list.split(',') {
                let dep = dep.trim();
                if !dep.is_empty() && !var_deps.contains(&dep.to_string()) {
                    var_deps.push(dep.to_string());
                }
            }
        }

        if !var_deps.is_empty() {
            deps.insert(var_name.clone(), var_deps);
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_build_sbt() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("build.sbt"),
            "ThisBuild / scalaVersion := \"3.3.1\"\n",
        )
        .unwrap();
        assert!(SbtResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_no_sbt() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!SbtResolver.detect(dir.path()));
    }

    #[test]
    fn test_parse_sbt_projects() {
        let content = r#"
ThisBuild / scalaVersion := "3.3.1"

lazy val common = (project in file("common"))

lazy val core = (project in file("core"))
  .dependsOn(common)

lazy val api = (project in file("api"))
  .dependsOn(core, common)
"#;
        let projects = parse_sbt_projects(content);
        assert_eq!(projects.len(), 3);
        assert!(projects.contains(&("common".to_string(), "common".to_string())));
        assert!(projects.contains(&("core".to_string(), "core".to_string())));
        assert!(projects.contains(&("api".to_string(), "api".to_string())));
    }

    #[test]
    fn test_parse_sbt_project_without_file() {
        let content = "lazy val core = project\n";
        let projects = parse_sbt_projects(content);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0], ("core".to_string(), "core".to_string()));
    }

    #[test]
    fn test_parse_sbt_depends_on() {
        let content = r#"
lazy val common = (project in file("common"))

lazy val core = (project in file("core"))
  .dependsOn(common)

lazy val api = (project in file("api"))
  .dependsOn(core, common)
"#;
        let deps = parse_sbt_dependencies(content);

        assert!(!deps.contains_key("common"));

        let core_deps = deps.get("core").unwrap();
        assert_eq!(core_deps, &vec!["common".to_string()]);

        let api_deps = deps.get("api").unwrap();
        assert!(api_deps.contains(&"core".to_string()));
        assert!(api_deps.contains(&"common".to_string()));
        assert_eq!(api_deps.len(), 2);
    }

    #[test]
    fn test_parse_sbt_chained_depends_on() {
        let content = r#"
lazy val common = (project in file("common"))

lazy val core = (project in file("core"))

lazy val api = (project in file("api"))
  .dependsOn(common).dependsOn(core)
"#;
        let deps = parse_sbt_dependencies(content);
        let api_deps = deps.get("api").unwrap();
        assert!(api_deps.contains(&"common".to_string()));
        assert!(api_deps.contains(&"core".to_string()));
        assert_eq!(api_deps.len(), 2);
    }

    #[test]
    fn test_resolve_sbt_project() {
        let dir = tempfile::tempdir().unwrap();

        let build_sbt = r#"
ThisBuild / scalaVersion := "3.3.1"
ThisBuild / version      := "0.1.0"

lazy val common = (project in file("common"))

lazy val core = (project in file("core"))
  .dependsOn(common)

lazy val api = (project in file("api"))
  .dependsOn(core, common)

lazy val root = (project in file("."))
  .aggregate(common, core, api)
"#;

        std::fs::write(dir.path().join("build.sbt"), build_sbt).unwrap();

        // Create project directories
        std::fs::create_dir_all(dir.path().join("common")).unwrap();
        std::fs::create_dir_all(dir.path().join("core")).unwrap();
        std::fs::create_dir_all(dir.path().join("api")).unwrap();

        let graph = SbtResolver.resolve(dir.path()).unwrap();

        // root maps to "." which is the tempdir itself, so it should also resolve
        assert!(graph.packages.contains_key(&PackageId("common".into())));
        assert!(graph.packages.contains_key(&PackageId("core".into())));
        assert!(graph.packages.contains_key(&PackageId("api".into())));
        assert!(graph.packages.contains_key(&PackageId("root".into())));
        assert_eq!(graph.packages.len(), 4);

        // core depends on common
        assert!(graph
            .edges
            .contains(&(PackageId("core".into()), PackageId("common".into()))));

        // api depends on core and common
        assert!(graph
            .edges
            .contains(&(PackageId("api".into()), PackageId("core".into()))));
        assert!(graph
            .edges
            .contains(&(PackageId("api".into()), PackageId("common".into()))));
    }

    #[test]
    fn test_test_command() {
        let cmd = SbtResolver.test_command(&PackageId("core".into()));
        assert_eq!(cmd, vec!["sbt", "core/test"]);
    }
}
