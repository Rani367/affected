use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

use crate::types::{Ecosystem, PackageConfig};

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub test: Option<TestConfig>,
    pub ignore: Option<Vec<String>>,
    pub packages: Option<HashMap<String, PackageConfig>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TestConfig {
    pub cargo: Option<String>,
    pub npm: Option<String>,
    pub go: Option<String>,
    pub python: Option<String>,
    pub maven: Option<String>,
    pub gradle: Option<String>,
    pub bun: Option<String>,
    pub dotnet: Option<String>,
    pub dart: Option<String>,
    pub swift: Option<String>,
    pub elixir: Option<String>,
    pub sbt: Option<String>,
}

impl Config {
    /// Load config from `.affected.toml` in the project root, or return defaults.
    pub fn load(root: &Path) -> Result<Self> {
        let config_path = root.join(".affected.toml");
        if config_path.exists() {
            debug!("Loading config from {}", config_path.display());
            let content = std::fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&content)?;
            debug!("Config loaded successfully");
            Ok(config)
        } else {
            debug!(
                "No .affected.toml found at {}, using defaults",
                root.display()
            );
            Ok(Self::default())
        }
    }

    /// Load config from a specific path (for --config flag).
    pub fn load_from(path: &Path) -> Result<Self> {
        debug!("Loading config from explicit path: {}", path.display());
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        debug!("Config loaded successfully from {}", path.display());
        Ok(config)
    }

    /// Get per-package configuration by name.
    pub fn package_config(&self, name: &str) -> Option<&PackageConfig> {
        self.packages.as_ref()?.get(name)
    }

    /// Get a custom test command for a given ecosystem and package.
    /// Replaces `{package}` placeholder with the actual package name.
    pub fn test_command_for(&self, ecosystem: Ecosystem, package: &str) -> Option<Vec<String>> {
        let template = match &self.test {
            Some(tc) => match ecosystem {
                Ecosystem::Cargo => tc.cargo.as_deref(),
                Ecosystem::Npm => tc.npm.as_deref(),
                Ecosystem::Go => tc.go.as_deref(),
                Ecosystem::Python => tc.python.as_deref(),
                Ecosystem::Yarn => tc.npm.as_deref(), // Yarn uses npm config
                Ecosystem::Maven => tc.maven.as_deref(),
                Ecosystem::Gradle => tc.gradle.as_deref(),
                Ecosystem::Bun => tc.bun.as_deref(),
                Ecosystem::Dotnet => tc.dotnet.as_deref(),
                Ecosystem::Dart => tc.dart.as_deref(),
                Ecosystem::Swift => tc.swift.as_deref(),
                Ecosystem::Elixir => tc.elixir.as_deref(),
                Ecosystem::Sbt => tc.sbt.as_deref(),
            },
            None => None,
        }?;

        let expanded = template.replace("{package}", package);
        Some(expanded.split_whitespace().map(String::from).collect())
    }

    /// Check if a file path matches any ignore patterns.
    pub fn is_ignored(&self, path: &str) -> bool {
        match &self.ignore {
            Some(patterns) => patterns.iter().any(|pat| {
                glob::Pattern::new(pat)
                    .map(|p| p.matches(path))
                    .unwrap_or(false)
            }),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_missing_config_returns_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load(dir.path()).unwrap();
        assert!(config.test.is_none());
        assert!(config.ignore.is_none());
    }

    #[test]
    fn test_load_valid_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".affected.toml"),
            r#"
ignore = ["*.md", "docs/**"]

[test]
cargo = "cargo nextest run -p {package}"
npm = "pnpm test --filter {package}"
"#,
        )
        .unwrap();

        let config = Config::load(dir.path()).unwrap();
        let tc = config.test.unwrap();
        assert!(tc.cargo.is_some());
        assert!(tc.npm.is_some());
        assert!(tc.go.is_none());
        assert!(tc.python.is_none());
        assert_eq!(config.ignore.unwrap().len(), 2);
    }

    #[test]
    fn test_load_invalid_toml_errors() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".affected.toml"), "this is [not valid toml").unwrap();
        assert!(Config::load(dir.path()).is_err());
    }

    #[test]
    fn test_command_for_with_placeholder() {
        let config = Config {
            test: Some(TestConfig {
                cargo: Some("cargo nextest run -p {package}".into()),
                ..TestConfig::default()
            }),
            ignore: None,
            packages: None,
        };

        let cmd = config
            .test_command_for(Ecosystem::Cargo, "my-crate")
            .unwrap();
        assert_eq!(cmd, vec!["cargo", "nextest", "run", "-p", "my-crate"]);
    }

    #[test]
    fn test_command_for_missing_ecosystem() {
        let config = Config {
            test: Some(TestConfig {
                cargo: Some("cargo test -p {package}".into()),
                ..TestConfig::default()
            }),
            ignore: None,
            packages: None,
        };

        assert!(config.test_command_for(Ecosystem::Npm, "pkg").is_none());
        assert!(config.test_command_for(Ecosystem::Go, "pkg").is_none());
        assert!(config.test_command_for(Ecosystem::Python, "pkg").is_none());
    }

    #[test]
    fn test_command_for_no_test_config() {
        let config = Config::default();
        assert!(config.test_command_for(Ecosystem::Cargo, "pkg").is_none());
    }

    #[test]
    fn test_is_ignored_md_files() {
        let config = Config {
            test: None,
            ignore: Some(vec!["*.md".into()]),
            packages: None,
        };

        assert!(config.is_ignored("README.md"));
        assert!(config.is_ignored("CHANGELOG.md"));
        assert!(!config.is_ignored("src/main.rs"));
    }

    #[test]
    fn test_is_ignored_glob_patterns() {
        let config = Config {
            test: None,
            ignore: Some(vec!["docs/**".into(), "*.txt".into()]),
            packages: None,
        };

        assert!(config.is_ignored("docs/guide.html"));
        assert!(config.is_ignored("docs/api/ref.md"));
        assert!(config.is_ignored("notes.txt"));
        assert!(!config.is_ignored("src/lib.rs"));
    }

    #[test]
    fn test_is_ignored_no_patterns() {
        let config = Config::default();
        assert!(!config.is_ignored("anything.rs"));
    }

    #[test]
    fn test_package_config_lookup() {
        let mut packages = HashMap::new();
        packages.insert(
            "my-pkg".to_string(),
            PackageConfig {
                test: Some("cargo nextest run -p my-pkg".to_string()),
                timeout: Some(120),
                skip: Some(false),
            },
        );
        let config = Config {
            test: None,
            ignore: None,
            packages: Some(packages),
        };

        let pc = config.package_config("my-pkg").unwrap();
        assert_eq!(pc.timeout, Some(120));
        assert!(config.package_config("nonexistent").is_none());
    }

    #[test]
    fn test_load_from_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let custom_path = dir.path().join("custom-config.toml");
        std::fs::write(
            &custom_path,
            r#"
ignore = ["*.lock"]
"#,
        )
        .unwrap();

        let config = Config::load_from(&custom_path).unwrap();
        assert_eq!(config.ignore.unwrap().len(), 1);
    }

    #[test]
    fn test_load_config_with_packages() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".affected.toml"),
            r#"
[packages.my-crate]
test = "cargo nextest run -p my-crate"
timeout = 60
skip = false
"#,
        )
        .unwrap();

        let config = Config::load(dir.path()).unwrap();
        let pc = config.package_config("my-crate").unwrap();
        assert_eq!(pc.test.as_deref(), Some("cargo nextest run -p my-crate"));
        assert_eq!(pc.timeout, Some(60));
        assert_eq!(pc.skip, Some(false));
    }
}
