use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use crate::resolvers::{file_to_package, Resolver};
use crate::types::{Ecosystem, Package, PackageId, ProjectGraph};

/// DotnetResolver detects .NET solutions via `*.sln` files and resolves project references.
///
/// Uses `glob` for solution file discovery, `regex` for parsing `.sln` project entries,
/// and `quick-xml` for parsing `<ProjectReference>` elements from `.csproj`/`.fsproj`/`.vbproj` files.
pub struct DotnetResolver;

impl Resolver for DotnetResolver {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Dotnet
    }

    fn detect(&self, root: &Path) -> bool {
        let pattern = root.join("*.sln").to_string_lossy().to_string();
        glob::glob(&pattern)
            .map(|mut paths| paths.any(|p| p.is_ok()))
            .unwrap_or(false)
    }

    fn resolve(&self, root: &Path) -> Result<ProjectGraph> {
        // Find the first .sln file at root
        let pattern = root.join("*.sln").to_string_lossy().to_string();
        let sln_path = glob::glob(&pattern)
            .context("Failed to glob for .sln files")?
            .filter_map(|p| p.ok())
            .next()
            .context("No .sln file found")?;

        let sln_content = std::fs::read_to_string(&sln_path)
            .with_context(|| format!("Failed to read {}", sln_path.display()))?;

        let sln_projects = parse_sln_projects(&sln_content)?;

        tracing::debug!(
            "Dotnet: found {} project entries in {}",
            sln_projects.len(),
            sln_path.display()
        );

        let mut packages = HashMap::new();
        // Map from normalized project-file path (relative to root) to PackageId
        let mut proj_path_to_id: HashMap<String, PackageId> = HashMap::new();

        for (name, rel_proj_path) in &sln_projects {
            let proj_file = root.join(rel_proj_path);
            if !proj_file.exists() {
                tracing::debug!(
                    "Dotnet: project file '{}' does not exist, skipping",
                    proj_file.display()
                );
                continue;
            }

            // The PackageId is the project's directory relative to root
            let proj_dir = proj_file.parent().unwrap_or(root);
            let rel_dir = proj_dir
                .strip_prefix(root)
                .unwrap_or(proj_dir)
                .to_string_lossy()
                .replace('\\', "/");
            let pkg_id = PackageId(rel_dir.clone());

            tracing::debug!(
                "Dotnet: discovered project '{}' at '{}'",
                name,
                rel_proj_path
            );

            proj_path_to_id.insert(rel_proj_path.clone(), pkg_id.clone());
            packages.insert(
                pkg_id.clone(),
                Package {
                    id: pkg_id,
                    name: name.clone(),
                    version: None,
                    path: proj_dir.to_path_buf(),
                    manifest_path: proj_file,
                },
            );
        }

        // Build dependency edges from ProjectReference elements
        let mut edges = Vec::new();

        for (_, rel_proj_path) in &sln_projects {
            let proj_file = root.join(rel_proj_path);
            if !proj_file.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&proj_file)?;
            let references = parse_project_references(&content)?;

            let from_id = match proj_path_to_id.get(rel_proj_path) {
                Some(id) => id.clone(),
                None => continue,
            };

            let proj_dir = proj_file.parent().unwrap_or(root);

            for ref_path in &references {
                // Resolve the reference path relative to the project file's directory
                let resolved = proj_dir.join(ref_path);
                let resolved = resolved
                    .canonicalize()
                    .unwrap_or(resolved)
                    .to_string_lossy()
                    .replace('\\', "/");

                // Try to match against known project paths
                for (known_path, to_id) in &proj_path_to_id {
                    let known_abs = root
                        .join(known_path)
                        .canonicalize()
                        .unwrap_or_else(|_| root.join(known_path))
                        .to_string_lossy()
                        .replace('\\', "/");
                    if resolved == known_abs {
                        edges.push((from_id.clone(), to_id.clone()));
                        break;
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
        vec![
            "dotnet".into(),
            "test".into(),
            package_id.0.clone(),
        ]
    }
}

/// Parse a `.sln` file's content and extract project entries.
///
/// Returns a vec of (project_name, normalized_relative_path) for `.csproj`, `.fsproj`, and `.vbproj` files.
fn parse_sln_projects(sln_content: &str) -> Result<Vec<(String, String)>> {
    let re = Regex::new(
        r#"Project\("[^"]*"\)\s*=\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"[^"]*""#,
    )
    .context("Failed to compile .sln regex")?;

    let mut projects = Vec::new();
    for line in sln_content.lines() {
        if let Some(caps) = re.captures(line) {
            let name = caps[1].to_string();
            let path = caps[2].replace('\\', "/");

            if path.ends_with(".csproj")
                || path.ends_with(".fsproj")
                || path.ends_with(".vbproj")
            {
                projects.push((name, path));
            }
        }
    }

    Ok(projects)
}

/// Parse a project file (`.csproj`/`.fsproj`/`.vbproj`) and extract `<ProjectReference Include="...">` paths.
///
/// Returns normalized paths (backslashes replaced with forward slashes).
fn parse_project_references(xml: &str) -> Result<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut references = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "ProjectReference" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        if key == "Include" {
                            let value = String::from_utf8_lossy(&attr.value)
                                .replace('\\', "/");
                            references.push(value);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Error parsing project file XML: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(references)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sln() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("MyApp.sln"),
            "Microsoft Visual Studio Solution File\n",
        )
        .unwrap();

        assert!(DotnetResolver.detect(dir.path()));
    }

    #[test]
    fn test_detect_no_sln() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!DotnetResolver.detect(dir.path()));
    }

    #[test]
    fn test_parse_sln_projects() {
        let sln = r#"
Microsoft Visual Studio Solution File, Format Version 12.00
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "Core", "src\Core\Core.csproj", "{AAA-BBB}"
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "Api", "src\Api\Api.csproj", "{CCC-DDD}"
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "SolutionFolder", "src\Folder", "{EEE-FFF}"
"#;

        let projects = parse_sln_projects(sln).unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].0, "Core");
        assert_eq!(projects[0].1, "src/Core/Core.csproj");
        assert_eq!(projects[1].0, "Api");
        assert_eq!(projects[1].1, "src/Api/Api.csproj");
    }

    #[test]
    fn test_parse_csproj_references() {
        let xml = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="..\Core\Core.csproj" />
    <PackageReference Include="Newtonsoft.Json" Version="13.0.1" />
  </ItemGroup>
</Project>"#;

        let refs = parse_project_references(xml).unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0], "../Core/Core.csproj");
    }

    #[test]
    fn test_resolve_dotnet_solution() {
        let dir = tempfile::tempdir().unwrap();

        // .sln file with 3 projects: Core, Api, Tests
        std::fs::write(
            dir.path().join("MyApp.sln"),
            r#"Microsoft Visual Studio Solution File, Format Version 12.00
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "Core", "src/Core/Core.csproj", "{AAA}"
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "Api", "src/Api/Api.csproj", "{BBB}"
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "Tests", "tests/Tests/Tests.csproj", "{CCC}"
"#,
        )
        .unwrap();

        // Core project (no internal deps)
        std::fs::create_dir_all(dir.path().join("src/Core")).unwrap();
        std::fs::write(
            dir.path().join("src/Core/Core.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
        )
        .unwrap();

        // Api depends on Core
        std::fs::create_dir_all(dir.path().join("src/Api")).unwrap();
        std::fs::write(
            dir.path().join("src/Api/Api.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="../Core/Core.csproj" />
  </ItemGroup>
</Project>"#,
        )
        .unwrap();

        // Tests depends on Api
        std::fs::create_dir_all(dir.path().join("tests/Tests")).unwrap();
        std::fs::write(
            dir.path().join("tests/Tests/Tests.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="../../src/Api/Api.csproj" />
  </ItemGroup>
</Project>"#,
        )
        .unwrap();

        let graph = DotnetResolver.resolve(dir.path()).unwrap();
        assert_eq!(graph.packages.len(), 3);
        assert!(graph.packages.contains_key(&PackageId("src/Core".into())));
        assert!(graph.packages.contains_key(&PackageId("src/Api".into())));
        assert!(graph
            .packages
            .contains_key(&PackageId("tests/Tests".into())));

        // Api depends on Core
        assert!(graph.edges.contains(&(
            PackageId("src/Api".into()),
            PackageId("src/Core".into()),
        )));
        // Tests depends on Api
        assert!(graph.edges.contains(&(
            PackageId("tests/Tests".into()),
            PackageId("src/Api".into()),
        )));
    }

    #[test]
    fn test_resolve_no_internal_deps() {
        let dir = tempfile::tempdir().unwrap();

        std::fs::write(
            dir.path().join("MyApp.sln"),
            r#"Microsoft Visual Studio Solution File, Format Version 12.00
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "Alpha", "src/Alpha/Alpha.csproj", "{AAA}"
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "Beta", "src/Beta/Beta.csproj", "{BBB}"
"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("src/Alpha")).unwrap();
        std::fs::write(
            dir.path().join("src/Alpha/Alpha.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.1" />
  </ItemGroup>
</Project>"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("src/Beta")).unwrap();
        std::fs::write(
            dir.path().join("src/Beta/Beta.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <PackageReference Include="Serilog" Version="3.0.0" />
  </ItemGroup>
</Project>"#,
        )
        .unwrap();

        let graph = DotnetResolver.resolve(dir.path()).unwrap();
        assert_eq!(graph.packages.len(), 2);
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_resolve_normalizes_backslashes() {
        let dir = tempfile::tempdir().unwrap();

        // .sln with Windows-style backslash paths
        std::fs::write(
            dir.path().join("App.sln"),
            "Microsoft Visual Studio Solution File, Format Version 12.00\r\n\
             Project(\"{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}\") = \"Lib\", \"src\\Lib\\Lib.csproj\", \"{AAA}\"\r\n\
             Project(\"{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}\") = \"App\", \"src\\App\\App.csproj\", \"{BBB}\"\r\n",
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("src/Lib")).unwrap();
        std::fs::write(
            dir.path().join("src/Lib/Lib.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join("src/App")).unwrap();
        std::fs::write(
            dir.path().join("src/App/App.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <ProjectReference Include="..\Lib\Lib.csproj" />
  </ItemGroup>
</Project>"#,
        )
        .unwrap();

        let graph = DotnetResolver.resolve(dir.path()).unwrap();
        assert_eq!(graph.packages.len(), 2);
        assert!(graph.packages.contains_key(&PackageId("src/Lib".into())));
        assert!(graph.packages.contains_key(&PackageId("src/App".into())));

        // App depends on Lib
        assert!(graph.edges.contains(&(
            PackageId("src/App".into()),
            PackageId("src/Lib".into()),
        )));
    }

    #[test]
    fn test_test_command() {
        let cmd = DotnetResolver.test_command(&PackageId("src/Core".into()));
        assert_eq!(cmd, vec!["dotnet", "test", "src/Core"]);
    }
}
