use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::resolvers::{file_to_package, Resolver};
use crate::types::{Ecosystem, Package, PackageId, ProjectGraph};

/// ElixirResolver detects Elixir Mix umbrella projects via `mix.exs` + `apps/` directory.
///
/// Scans `apps/*/mix.exs` to discover sub-applications and their umbrella dependencies.
pub struct ElixirResolver;

impl Resolver for ElixirResolver {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Elixir
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("mix.exs").exists() && root.join("apps").is_dir()
    }

    fn resolve(&self, root: &Path) -> Result<ProjectGraph> {
        let apps_dir = root.join("apps");

        // Discover app directories that contain a mix.exs
        let mut app_dirs = Vec::new();
        for entry in std::fs::read_dir(&apps_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.join("mix.exs").exists() {
                app_dirs.push(path);
            }
        }

        let mut packages = HashMap::new();
        let mut name_to_id: HashMap<String, PackageId> = HashMap::new();
        let mut app_contents: HashMap<PackageId, String> = HashMap::new();

        for app_dir in &app_dirs {
            let dir_name = app_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let mix_path = app_dir.join("mix.exs");
            let content = std::fs::read_to_string(&mix_path)?;

            let app_name = parse_app_name(&content, &dir_name);
            let rel_path = format!("apps/{}", dir_name);
            let pkg_id = PackageId(rel_path);

            tracing::debug!("Elixir: discovered app '{}' at {:?}", app_name, app_dir);

            name_to_id.insert(app_name.clone(), pkg_id.clone());
            app_contents.insert(pkg_id.clone(), content);
            packages.insert(
                pkg_id.clone(),
                Package {
                    id: pkg_id,
                    name: app_name,
                    version: None,
                    path: app_dir.clone(),
                    manifest_path: mix_path,
                },
            );
        }

        // Build dependency edges
        let known_apps: HashSet<&str> = name_to_id.keys().map(|s| s.as_str()).collect();
        let mut edges = Vec::new();

        for (pkg_id, content) in &app_contents {
            let deps = parse_umbrella_deps(content);
            for dep_name in &deps {
                if known_apps.contains(dep_name.as_str()) {
                    if let Some(to_id) = name_to_id.get(dep_name) {
                        edges.push((pkg_id.clone(), to_id.clone()));
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
        // package_id is "apps/<dir_name>"; extract the app directory name
        let app_name = package_id
            .0
            .strip_prefix("apps/")
            .unwrap_or(&package_id.0);
        vec![
            "mix".into(),
            "cmd".into(),
            "--app".into(),
            app_name.into(),
            "mix".into(),
            "test".into(),
        ]
    }
}

/// Extract the app name from mix.exs content.
///
/// Looks for `app: :some_name` and returns `"some_name"`.
/// Falls back to the directory name if no match is found.
fn parse_app_name(content: &str, dir_name: &str) -> String {
    let re = Regex::new(r"app:\s*:([\w]+)").expect("invalid regex");
    re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| dir_name.to_string())
}

/// Extract umbrella dependency names from mix.exs content.
///
/// Matches two patterns:
/// - `{:dep_name, in_umbrella: true}`
/// - `{:dep_name, path: "../dep_name"}`
fn parse_umbrella_deps(content: &str) -> Vec<String> {
    let mut deps = Vec::new();

    // Pattern 1: {:dep_name, in_umbrella: true}
    let re_umbrella =
        Regex::new(r#"\{:([\w]+),\s*in_umbrella:\s*true\}"#).expect("invalid regex");
    for caps in re_umbrella.captures_iter(content) {
        if let Some(m) = caps.get(1) {
            deps.push(m.as_str().to_string());
        }
    }

    // Pattern 2: {:dep_name, path: "../dep_name"}
    let re_path = Regex::new(r#"\{:([\w]+),.*?path:\s*""#).expect("invalid regex");
    for caps in re_path.captures_iter(content) {
        if let Some(m) = caps.get(1) {
            let name = m.as_str().to_string();
            if !deps.contains(&name) {
                deps.push(name);
            }
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_umbrella() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("mix.exs"),
            r#"defmodule MyUmbrella.MixProject do
  use Mix.Project

  def project do
    [apps_path: "apps"]
  end
end"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("apps")).unwrap();

        assert!(ElixirResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_no_apps_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("mix.exs"),
            r#"defmodule MyApp.MixProject do
  use Mix.Project
end"#,
        )
        .unwrap();

        assert!(!ElixirResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_no_elixir() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!ElixirResolver.detect(dir.path()));
    }

    #[test]
    fn test_parse_app_name() {
        let content = r#"defmodule Api.MixProject do
  use Mix.Project

  def project do
    [
      app: :api,
      version: "0.1.0",
      deps: deps()
    ]
  end
end"#;

        assert_eq!(parse_app_name(content, "fallback"), "api");
    }

    #[test]
    fn test_parse_app_name_fallback() {
        let content = r#"defmodule Weird.MixProject do
  use Mix.Project
end"#;

        assert_eq!(parse_app_name(content, "my_dir"), "my_dir");
    }

    #[test]
    fn test_parse_umbrella_deps() {
        let content = r#"defmodule Api.MixProject do
  use Mix.Project

  def project do
    [
      app: :api,
      version: "0.1.0",
      deps: deps()
    ]
  end

  defp deps do
    [
      {:core, in_umbrella: true},
      {:shared, in_umbrella: true},
      {:phoenix, "~> 1.7"},
      {:utils, path: "../utils"}
    ]
  end
end"#;

        let deps = parse_umbrella_deps(content);
        assert_eq!(deps, vec!["core", "shared", "utils"]);
    }

    #[test]
    fn test_resolve_umbrella_project() {
        let dir = tempfile::tempdir().unwrap();

        // Root mix.exs
        std::fs::write(
            dir.path().join("mix.exs"),
            r#"defmodule MyUmbrella.MixProject do
  use Mix.Project

  def project do
    [apps_path: "apps"]
  end
end"#,
        )
        .unwrap();

        // apps/ directory with three apps: core, shared, api
        let apps = dir.path().join("apps");
        std::fs::create_dir_all(apps.join("core")).unwrap();
        std::fs::write(
            apps.join("core/mix.exs"),
            r#"defmodule Core.MixProject do
  use Mix.Project

  def project do
    [
      app: :core,
      version: "0.1.0",
      deps: deps()
    ]
  end

  defp deps do
    [
      {:jason, "~> 1.4"}
    ]
  end
end"#,
        )
        .unwrap();

        std::fs::create_dir_all(apps.join("shared")).unwrap();
        std::fs::write(
            apps.join("shared/mix.exs"),
            r#"defmodule Shared.MixProject do
  use Mix.Project

  def project do
    [
      app: :shared,
      version: "0.1.0",
      deps: deps()
    ]
  end

  defp deps do
    [
      {:core, in_umbrella: true}
    ]
  end
end"#,
        )
        .unwrap();

        std::fs::create_dir_all(apps.join("api")).unwrap();
        std::fs::write(
            apps.join("api/mix.exs"),
            r#"defmodule Api.MixProject do
  use Mix.Project

  def project do
    [
      app: :api,
      version: "0.1.0",
      deps: deps()
    ]
  end

  defp deps do
    [
      {:core, in_umbrella: true},
      {:shared, in_umbrella: true},
      {:phoenix, "~> 1.7"}
    ]
  end
end"#,
        )
        .unwrap();

        let graph = ElixirResolver.resolve(dir.path()).unwrap();

        // 3 packages discovered
        assert_eq!(graph.packages.len(), 3);
        assert!(graph
            .packages
            .contains_key(&PackageId("apps/core".into())));
        assert!(graph
            .packages
            .contains_key(&PackageId("apps/shared".into())));
        assert!(graph.packages.contains_key(&PackageId("apps/api".into())));

        // Check package names
        assert_eq!(
            graph.packages[&PackageId("apps/core".into())].name,
            "core"
        );
        assert_eq!(
            graph.packages[&PackageId("apps/shared".into())].name,
            "shared"
        );
        assert_eq!(
            graph.packages[&PackageId("apps/api".into())].name,
            "api"
        );

        // shared depends on core
        assert!(graph.edges.contains(&(
            PackageId("apps/shared".into()),
            PackageId("apps/core".into()),
        )));

        // api depends on core and shared
        assert!(graph.edges.contains(&(
            PackageId("apps/api".into()),
            PackageId("apps/core".into()),
        )));
        assert!(graph.edges.contains(&(
            PackageId("apps/api".into()),
            PackageId("apps/shared".into()),
        )));

        // core has no internal deps
        let core_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|(from, _)| from == &PackageId("apps/core".into()))
            .collect();
        assert!(core_edges.is_empty());
    }

    #[test]
    fn test_test_command() {
        let cmd = ElixirResolver.test_command(&PackageId("apps/api".into()));
        assert_eq!(cmd, vec!["mix", "cmd", "--app", "api", "mix", "test"]);
    }
}
