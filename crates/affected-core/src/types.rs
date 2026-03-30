use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Unique identifier for a package within a project.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PackageId(pub String);

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A single package/module discovered by a resolver.
#[derive(Debug, Clone)]
pub struct Package {
    pub id: PackageId,
    pub name: String,
    pub version: Option<String>,
    /// Absolute path to the package root directory.
    pub path: PathBuf,
    /// Absolute path to the manifest file.
    pub manifest_path: PathBuf,
}

/// The fully-resolved project graph returned by a resolver.
#[derive(Debug)]
pub struct ProjectGraph {
    pub packages: HashMap<PackageId, Package>,
    /// Dependency edges: (dependent, dependency). "A depends on B" = (A, B).
    pub edges: Vec<(PackageId, PackageId)>,
    pub root: PathBuf,
}

/// What kind of ecosystem was detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Ecosystem {
    Cargo,
    Npm,
    Go,
    Python,
    Yarn,
    Maven,
    Gradle,
    Bun,
    Dotnet,
    Dart,
    Swift,
    Elixir,
    Sbt,
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ecosystem::Cargo => write!(f, "cargo"),
            Ecosystem::Npm => write!(f, "npm"),
            Ecosystem::Go => write!(f, "go"),
            Ecosystem::Python => write!(f, "python"),
            Ecosystem::Yarn => write!(f, "yarn"),
            Ecosystem::Maven => write!(f, "maven"),
            Ecosystem::Gradle => write!(f, "gradle"),
            Ecosystem::Bun => write!(f, "bun"),
            Ecosystem::Dotnet => write!(f, "dotnet"),
            Ecosystem::Dart => write!(f, "dart"),
            Ecosystem::Swift => write!(f, "swift"),
            Ecosystem::Elixir => write!(f, "elixir"),
            Ecosystem::Sbt => write!(f, "sbt"),
        }
    }
}

/// An explanation of why a package was affected.
#[derive(Debug, Serialize)]
pub struct ExplainEntry {
    pub package: String,
    pub reason: ExplainReason,
}

/// The reason a package is affected: either directly changed or transitively affected.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ExplainReason {
    DirectlyChanged { files: Vec<String> },
    TransitivelyAffected { chain: Vec<String> },
}

/// The result of the "affected" computation.
#[derive(Debug, Serialize)]
pub struct AffectedResult {
    pub affected: Vec<String>,
    pub base: String,
    pub changed_files: usize,
    pub total_packages: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanations: Option<Vec<ExplainEntry>>,
}

/// A single test result in JSON output format.
#[derive(Debug, Serialize)]
pub struct TestResultJson {
    pub package: String,
    pub success: bool,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
}

/// Summary of test results in JSON output format.
#[derive(Debug, Serialize)]
pub struct TestSummaryJson {
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub duration_ms: u64,
}

/// Full JSON output for test results.
#[derive(Debug, Serialize)]
pub struct TestOutputJson {
    pub affected: Vec<String>,
    pub results: Vec<TestResultJson>,
    pub summary: TestSummaryJson,
}

/// Per-package configuration from `.affected.toml`.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct PackageConfig {
    pub test: Option<String>,
    pub timeout: Option<u64>,
    pub skip: Option<bool>,
}
