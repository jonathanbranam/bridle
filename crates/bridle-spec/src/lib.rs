//! Parser for capability spec files (`design/specs/<capability>.md`).
//!
//! The grammar is data-contracts' `tools/spec-to-feature.py` grammar plus
//! bridle's inline ids and flags (`docs/design/specs.md`, `traceability.md`):
//!
//! ```markdown
//! ### Requirement: <title>   {#r-7fa2 protected traces=a-12cd@3f9e}
//! <SHALL text>
//! #### Scenario: <title>     {#s-b310}
//! *Verification*: **executable** @tag
//! - **GIVEN** ...
//! ```
//!
//! Parsing is strict: near-misses are rejected, never repaired, and every
//! problem in the file is reported, not just the first. The crate is pure: no
//! async, no daemon, no I/O beyond [`parse_file`].

mod ids;
mod parse;

pub use ids::{Assigned, assign_ids};

use std::fmt;
use std::path::Path;

/// A parsed capability spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    /// The text of a `# Title` heading, if the file has one.
    pub title: Option<String>,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub title: String,
    /// `r-xxxx`, absent until `bridle spec id` assigns it.
    pub id: Option<String>,
    /// The `protected` flag: changing it goes through the human plan gate.
    pub protected: bool,
    pub traces: Vec<Trace>,
    /// The prose between the heading and the first scenario, trimmed.
    pub text: String,
    /// 1-based line of the heading.
    pub line: usize,
    /// Never empty in a successfully parsed spec.
    pub scenarios: Vec<Scenario>,
}

/// An upward link, `traces=a-12cd@3f9e`: the upstream id and the hash of its
/// text when the link was last confirmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trace {
    pub target: String,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub title: String,
    /// `s-xxxx`, absent until assigned.
    pub id: Option<String>,
    /// 1-based line of the heading.
    pub line: usize,
    pub verification: Verification,
    /// `@tag`s from the Verification line, without the `@`.
    pub tags: Vec<String>,
    /// Free prose between the Verification line and the first step
    /// (executable scenarios only), lines joined by `\n`.
    pub description: String,
    /// Empty for a non-executable scenario, whose body is prose nobody parses.
    pub steps: Vec<Step>,
    pub examples: Option<Examples>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verification {
    Executable,
    NonExecutable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub keyword: Keyword,
    /// Continuation lines joined with a space, `\<` `\>` unescaped.
    pub text: String,
    /// 1-based line of the step's bullet.
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Given,
    When,
    Then,
    And,
    But,
}

impl Keyword {
    /// The spelling used between `**` in a spec file.
    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::Given => "GIVEN",
            Keyword::When => "WHEN",
            Keyword::Then => "THEN",
            Keyword::And => "AND",
            Keyword::But => "BUT",
        }
    }
}

/// The `*Examples*:` table of a scenario outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Examples {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// One problem in a spec file. `line` and `column` are 1-based (column counts
/// characters); `message` says what was expected and what was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.file, self.line, self.column, self.message
        )
    }
}

/// Why [`parse_file`] failed.
#[derive(Debug, thiserror::Error)]
pub enum ParseFileError {
    #[error("reading {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("{} spec diagnostic(s)", .0.len())]
    Diagnostics(Vec<Diagnostic>),
}

/// Parse spec text. `file` is only used to label diagnostics. Returns every
/// diagnostic found, in file order, or the spec if there are none.
pub fn parse_str(file: &str, text: &str) -> Result<Spec, Vec<Diagnostic>> {
    parse::parse(file, text)
}

/// Read and parse a spec file.
pub fn parse_file(path: &Path) -> Result<Spec, ParseFileError> {
    let text = std::fs::read_to_string(path).map_err(|source| ParseFileError::Io {
        path: path.display().to_string(),
        source,
    })?;
    parse_str(&path.display().to_string(), &text).map_err(ParseFileError::Diagnostics)
}
