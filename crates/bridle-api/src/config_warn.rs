//! Lenient config reading: unknown sections and keys are accepted and reported as warnings,
//! so an older binary can read a file a newer one wrote (docs/design/agent-host/daemon.md,
//! "Config warnings"). Wrong types and missing required keys stay hard errors.
//!
//! Loaders call [`parse`] instead of `toml::from_str`. The structs are the real ones (no
//! `deny_unknown_fields`), so the check cannot drift from what the binary reads.

use std::collections::BTreeSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use serde::de::value::{Error as ValueError, StrDeserializer};
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor,
};
use serde_ignored::Path;

/// Where [`report`] sends a finding the first time this process sees it.
const LOG: u8 = 0;
const STDERR: u8 = 1;
const QUIET: u8 = 2;
static MODE: AtomicU8 = AtomicU8::new(LOG);

/// Every finding this process has reported, in first-seen order.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The CLI prints each finding to stderr, once per process run.
pub fn print_to_stderr() {
    MODE.store(STDERR, Ordering::Relaxed);
}

/// Collect findings without printing them (doctor prints its own report).
pub fn collect_quietly() {
    MODE.store(QUIET, Ordering::Relaxed);
}

/// Every finding reported so far in this process.
pub fn seen() -> Vec<String> {
    SEEN.lock().map(|s| s.clone()).unwrap_or_default()
}

/// Records `findings`; each one not yet seen in this process is logged (the daemon) or
/// printed to stderr (the CLI). A reload that finds the same set says nothing new.
pub fn report(findings: &[String]) {
    let Ok(mut seen) = SEEN.lock() else { return };
    for f in findings {
        if seen.contains(f) {
            continue;
        }
        seen.push(f.clone());
        match MODE.load(Ordering::Relaxed) {
            STDERR => eprintln!("warning: {f}"),
            QUIET => {}
            _ => tracing::warn!("{f}"),
        }
    }
}

/// Parses `text` as `T`, returning the value and the findings for what `T` does not know.
/// `file` is the path as read, for the message.
pub fn parse_with_findings<T: DeserializeOwned>(
    text: &str,
    file: &str,
) -> Result<(T, Vec<String>), toml::de::Error> {
    let mut ignored: Vec<Vec<Step>> = Vec::new();
    let de = toml::Deserializer::parse(text)?;
    let value: T = serde_ignored::deserialize(de, |path| ignored.push(steps(&path)))?;
    if ignored.is_empty() {
        return Ok((value, Vec::new()));
    }
    let doc: toml::Table = toml::from_str(text)?;
    let version = env!("CARGO_PKG_VERSION");
    // Sorted so the output does not depend on the order serde walks the file.
    let findings: BTreeSet<String> = ignored
        .iter()
        .map(|steps| finding::<T>(&doc, steps, file, version))
        .collect();
    let findings = findings.into_iter().collect();
    Ok((value, findings))
}

/// [`parse_with_findings`], then [`report`]s the findings. For the loaders.
pub fn parse<T: DeserializeOwned>(text: &str, file: &str) -> Result<T, toml::de::Error> {
    let (value, findings) = parse_with_findings(text, file)?;
    report(&findings);
    Ok(value)
}

/// [`parse`] for a struct that reads only some sections of a file shared with others (the
/// mail bridge reads `[mail]` out of `config.toml`): findings outside `own` are the other
/// readers' sections, not unknowns, and are dropped.
pub fn parse_partial<T: DeserializeOwned>(
    text: &str,
    file: &str,
    own: &[&str],
) -> Result<T, toml::de::Error> {
    let (value, findings) = parse_with_findings::<T>(text, file)?;
    let findings: Vec<String> = findings
        .into_iter()
        .filter(|f| {
            own.iter().any(|o| {
                let after = f.split_once(": unknown ").map_or("", |(_, r)| r);
                let after = after
                    .strip_prefix("section [")
                    .or_else(|| after.strip_prefix("key "))
                    .unwrap_or(after);
                after
                    .strip_prefix(o)
                    .is_some_and(|r| r.starts_with(['.', ']', ' ']))
            })
        })
        .collect();
    report(&findings);
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Key(String),
    Index(usize),
}

fn steps(path: &Path<'_>) -> Vec<Step> {
    let mut out = match path {
        Path::Root => Vec::new(),
        Path::Seq { parent, index } => {
            let mut v = steps(parent);
            v.push(Step::Index(*index));
            v
        }
        Path::Map { parent, key } => {
            let mut v = steps(parent);
            v.push(Step::Key(key.clone()));
            v
        }
        Path::Some { parent }
        | Path::NewtypeStruct { parent }
        | Path::NewtypeVariant { parent } => steps(parent),
    };
    out.shrink_to_fit();
    out
}

fn finding<T: DeserializeOwned>(
    doc: &toml::Table,
    steps: &[Step],
    file: &str,
    version: &str,
) -> String {
    let tail = format!("(unknown to this build (bridle {version}); a newer build may use it)");
    let dotted = steps
        .iter()
        .filter_map(|s| match s {
            Step::Key(k) => Some(k.as_str()),
            Step::Index(_) => None,
        })
        .collect::<Vec<_>>()
        .join(".");
    if is_table(doc, steps) {
        return format!("{file}: unknown section [{dotted}] {tail}");
    }
    let (Some(Step::Key(key)), parent) = (steps.last(), &steps[..steps.len().saturating_sub(1)])
    else {
        return format!("{file}: unknown key {dotted} {tail}");
    };
    let mut line = format!("{file}: unknown key {dotted} {tail}");
    if let Some(close) = known_fields::<T>(parent)
        .into_iter()
        .filter(|k| edit_distance(k, key) <= 2)
        .min_by_key(|k| edit_distance(k, key))
    {
        line.push_str(&format!(" - did you mean {close}?"));
    }
    line
}

fn is_table(doc: &toml::Table, steps: &[Step]) -> bool {
    let mut cur: Option<&toml::Value> = None;
    for s in steps {
        cur = match (s, cur) {
            (Step::Key(k), None) => doc.get(k),
            (Step::Key(k), Some(toml::Value::Table(t))) => t.get(k),
            (Step::Index(i), Some(toml::Value::Array(a))) => a.get(*i),
            _ => None,
        };
    }
    match cur {
        Some(toml::Value::Table(_)) => true,
        Some(toml::Value::Array(a)) => a.iter().all(toml::Value::is_table) && !a.is_empty(),
        _ => false,
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != cb);
            cur.push(sub.min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The field names the struct at `path` (below `T`) declares: `T`'s own `Deserialize` impl
/// is run against a stand-in deserializer that walks `path` and records the `fields` slice
/// serde hands to `deserialize_struct`.
fn known_fields<T: DeserializeOwned>(path: &[Step]) -> BTreeSet<String> {
    let found = std::cell::RefCell::new(BTreeSet::new());
    let _ = T::deserialize(Probe {
        path,
        found: &found,
    });
    found.into_inner()
}

struct Probe<'a> {
    path: &'a [Step],
    found: &'a std::cell::RefCell<BTreeSet<String>>,
}

impl<'de> Deserializer<'de> for Probe<'_> {
    type Error = ValueError;

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.path.split_first() {
            None => {
                self.found
                    .borrow_mut()
                    .extend(fields.iter().map(|f| (*f).to_string()));
                Err(de::Error::custom("probe done"))
            }
            Some(_) => self.deserialize_map(visitor),
        }
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.path.split_first() {
            Some((Step::Key(k), rest)) => visitor.visit_map(OneEntry {
                key: Some(k.as_str()),
                rest,
                found: self.found,
            }),
            _ => Err(de::Error::custom("probe: no such key")),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.path.split_first() {
            Some((Step::Index(_), rest)) => visitor.visit_seq(OneElement {
                rest: Some(rest),
                found: self.found,
            }),
            _ => Err(de::Error::custom("probe: no such index")),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(de::Error::custom("probe: unsupported shape"))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        unit unit_struct tuple tuple_struct enum identifier ignored_any
    }
}

struct OneEntry<'a> {
    key: Option<&'a str>,
    rest: &'a [Step],
    found: &'a std::cell::RefCell<BTreeSet<String>>,
}

impl<'de> MapAccess<'de> for OneEntry<'_> {
    type Error = ValueError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.key.take() {
            Some(k) => seed
                .deserialize(StrDeserializer::<ValueError>::new(k))
                .map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        seed.deserialize(Probe {
            path: self.rest,
            found: self.found,
        })
    }
}

struct OneElement<'a> {
    rest: Option<&'a [Step]>,
    found: &'a std::cell::RefCell<BTreeSet<String>>,
}

impl<'de> SeqAccess<'de> for OneElement<'_> {
    type Error = ValueError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        match self.rest.take() {
            Some(rest) => seed
                .deserialize(Probe {
                    path: rest,
                    found: self.found,
                })
                .map(Some),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    // An old-shaped config: built the way the real structs are, minus what a newer build added.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct OldRaw {
        budget: Option<OldBudget>,
        projects: Option<BTreeMap<String, OldProject>>,
    }
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct OldBudget {
        max_workers: Option<u32>,
    }
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct OldProject {
        port: u16,
    }

    const TAIL: &str = "(unknown to this build (bridle VERSION); a newer build may use it)";
    fn tail() -> String {
        TAIL.replace("VERSION", env!("CARGO_PKG_VERSION"))
    }

    #[test]
    fn newer_file_loads_with_exactly_the_two_findings() {
        let text = "[budget]\nmax_workers = 3\nnew_knob = 1\n\n[gateway]\nurl = \"x\"\n";
        let (raw, findings) = parse_with_findings::<OldRaw>(text, "config.toml").unwrap();
        assert_eq!(raw.budget.unwrap().max_workers, Some(3));
        assert_eq!(
            findings,
            vec![
                format!("config.toml: unknown key budget.new_knob {}", tail()),
                format!("config.toml: unknown section [gateway] {}", tail()),
            ]
        );
    }

    #[test]
    fn typo_gets_a_did_you_mean() {
        let text = "[budget]\nmax_worker = 3\n";
        let (_, findings) = parse_with_findings::<OldRaw>(text, "c.toml").unwrap();
        assert_eq!(
            findings,
            vec![format!(
                "c.toml: unknown key budget.max_worker {} - did you mean max_workers?",
                tail()
            )]
        );
    }

    #[test]
    fn nested_map_entry_key_and_section() {
        let text = "[projects.bridle]\nport = 1\nnope = 2\n[projects.bridle.extra]\nx = 1\n";
        let (_, findings) = parse_with_findings::<OldRaw>(text, "c.toml").unwrap();
        assert_eq!(
            findings,
            vec![
                format!("c.toml: unknown key projects.bridle.nope {}", tail()),
                format!("c.toml: unknown section [projects.bridle.extra] {}", tail()),
            ]
        );
    }

    #[test]
    fn wrong_type_still_errors() {
        let text = "[budget]\nmax_workers = \"three\"\n";
        assert!(parse_with_findings::<OldRaw>(text, "c.toml").is_err());
    }

    #[test]
    fn report_says_each_finding_once() {
        let f = vec!["report-once-test: unknown key a.b".to_string()];
        report(&f);
        report(&f);
        let n = seen().iter().filter(|s| **s == f[0]).count();
        assert_eq!(n, 1);
    }
}
