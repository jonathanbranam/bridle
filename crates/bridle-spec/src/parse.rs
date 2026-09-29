use std::collections::{BTreeSet, HashMap};

use crate::{
    Diagnostic, Examples, Keyword, Requirement, Scenario, Spec, Step, Trace, Verification,
};

const MARKER_LABEL: &str = "*Verification*:";
const VALUE_EXECUTABLE: &str = "**executable**";
const VALUE_NON_EXECUTABLE: &str = "**non-executable**";
const EXAMPLES_LABEL: &str = "*Examples*:";
const KEYWORDS: [Keyword; 5] = [
    Keyword::Given,
    Keyword::And,
    Keyword::When,
    Keyword::Then,
    Keyword::But,
];

struct Parser<'a> {
    file: &'a str,
    lines: Vec<&'a str>,
    diags: Vec<Diagnostic>,
    /// id -> line of first use, across requirements and scenarios.
    ids: HashMap<String, usize>,
}

pub(crate) fn parse(file: &str, text: &str) -> Result<Spec, Vec<Diagnostic>> {
    let mut p = Parser {
        file,
        lines: text.lines().collect(),
        diags: Vec::new(),
        ids: HashMap::new(),
    };
    let spec = p.run();
    if p.diags.is_empty() {
        Ok(spec)
    } else {
        Err(p.diags)
    }
}

/// 1-based character column of byte `offset` in `line`.
pub(crate) fn col(line: &str, offset: usize) -> usize {
    line[..offset].chars().count() + 1
}

/// Byte offset of `sub` (a subslice of `line`) within `line`.
pub(crate) fn offset_in(line: &str, sub: &str) -> usize {
    sub.as_ptr() as usize - line.as_ptr() as usize
}

fn indent_col(line: &str) -> usize {
    col(line, line.len() - line.trim_start().len())
}

/// A line that is recognisably an attempt at `label` (ignoring its asterisks
/// and case), so a near-miss is reported as one rather than as missing.
fn looks_like(line: &str, label: &str) -> bool {
    line.trim()
        .replace('*', "")
        .to_lowercase()
        .starts_with(&label.replace('*', "").to_lowercase())
}

fn is_tag_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub(crate) fn is_id(s: &str, prefix: char) -> bool {
    let mut it = s.chars();
    it.next() == Some(prefix) && it.next() == Some('-') && is_hex(it.as_str())
}

fn is_hex(s: &str) -> bool {
    s.len() >= 4 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

fn unescape(s: &str) -> String {
    s.replace("\\<", "<").replace("\\>", ">")
}

/// Names of unescaped `<placeholder>`s: an ASCII letter or `_`, then word
/// characters or `-`. `a < b` never matches.
fn bare_placeholders(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    for (i, _) in text.match_indices('<') {
        if text[..i].ends_with('\\') {
            continue;
        }
        let rest = &text[i + 1..];
        if let Some(end) = rest.find('>') {
            let name = &rest[..end];
            let mut cs = name.chars();
            let ok = cs
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && cs.all(|c| c.is_alphanumeric() || c == '_' || c == '-');
            if ok {
                out.push(&text[i..i + end + 2]);
            }
        }
    }
    out
}

/// Names inside `<...>` in already-unescaped text.
fn placeholder_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find('<') {
        let after = &rest[i + 1..];
        match after.find(['<', '>']) {
            Some(j) if after.as_bytes()[j] == b'>' && j > 0 => {
                out.push(after[..j].to_string());
                rest = &after[j + 1..];
            }
            Some(j) => rest = &after[j..],
            None => break,
        }
    }
    out
}

/// Attributes parsed from a heading's trailing `{...}`.
#[derive(Default)]
struct Attrs {
    id: Option<String>,
    protected: bool,
    traces: Vec<Trace>,
}

enum Heading<'a> {
    Requirement(&'a str),
    Scenario(&'a str),
    Other,
}

impl<'a> Parser<'a> {
    fn diag(&mut self, line: usize, column: usize, message: String) {
        self.diags.push(Diagnostic {
            file: self.file.to_string(),
            line: line + 1,
            column,
            message,
        });
    }

    /// Index of the next line starting with `#` after `i`, or the end.
    fn block_end(&self, i: usize) -> usize {
        (i + 1..self.lines.len())
            .find(|&j| self.lines[j].starts_with('#'))
            .unwrap_or(self.lines.len())
    }

    fn run(&mut self) -> Spec {
        let mut title = None;
        let mut reqs: Vec<Requirement> = Vec::new();
        let mut i = 0;
        while i < self.lines.len() {
            let line = self.lines[i];
            match self.classify(i, line) {
                Heading::Requirement(rest) => {
                    let end = self.block_end(i);
                    let req = self.requirement(i, rest, end);
                    reqs.push(req);
                }
                Heading::Scenario(rest) => {
                    let end = self.block_end(i);
                    if let Some(sc) = self.scenario(i, rest, end) {
                        match reqs.last_mut() {
                            Some(r) => r.scenarios.push(sc),
                            None => self.diag(
                                i,
                                1,
                                format!(
                                    "expected scenario {:?} under a '### Requirement:' heading, found it before any requirement",
                                    sc.title
                                ),
                            ),
                        }
                    }
                }
                Heading::Other => {
                    if title.is_none()
                        && let Some(t) = line.strip_prefix("# ")
                    {
                        title = Some(t.trim().to_string());
                    }
                }
            }
            i += 1;
        }
        for r in &reqs {
            if r.scenarios.is_empty() {
                self.diag(
                    r.line - 1,
                    1,
                    format!(
                        "expected requirement {:?} to have at least one '#### Scenario:', found none",
                        r.title
                    ),
                );
            }
        }
        // Diagnostics were pushed out of file order by the scenario-less check.
        self.diags.sort_by_key(|d| (d.line, d.column));
        Spec {
            title,
            requirements: reqs,
        }
    }

    /// Recognise requirement/scenario headings and reject near-misses of
    /// them (wrong level, missing space, wrong case).
    fn classify(&mut self, i: usize, line: &'a str) -> Heading<'a> {
        if !line.starts_with('#') {
            return Heading::Other;
        }
        if let Some(rest) = line.strip_prefix("### Requirement:") {
            return Heading::Requirement(rest);
        }
        if let Some(rest) = line.strip_prefix("#### Scenario:") {
            return Heading::Scenario(rest);
        }
        let level = line.chars().take_while(|&c| c == '#').count();
        let after = line[level..].trim_start();
        let lower = after.to_lowercase();
        let (exact, expected) = if lower.starts_with("requirement:") {
            (false, "### Requirement:")
        } else if lower.starts_with("scenario:") || (level == 4 && lower.starts_with("scenario")) {
            (false, "#### Scenario:")
        } else {
            (true, "")
        };
        if !exact {
            self.diag(
                i,
                1,
                format!(
                    "expected heading '{expected} <title>', found {:?}",
                    line.trim()
                ),
            );
        }
        Heading::Other
    }

    /// Split `rest` (text after the `Requirement:`/`Scenario:` label) into a
    /// title and attributes. `kind` is the id prefix, `r` or `s`.
    fn title_and_attrs(&mut self, i: usize, rest: &str, kind: char) -> (String, Attrs) {
        let line = self.lines[i];
        let rest = rest.trim();
        let mut attrs = Attrs::default();
        let title = if rest.ends_with('}') {
            match rest.rfind('{') {
                Some(open) => {
                    let inner = &rest[open + 1..rest.len() - 1];
                    self.attrs(i, inner, kind, &mut attrs);
                    rest[..open].trim_end()
                }
                None => {
                    let c = col(line, offset_in(line, rest));
                    self.diag(
                        i,
                        c,
                        format!("expected '{{' to open the trailing '}}', found {rest:?}"),
                    );
                    rest
                }
            }
        } else if let Some(open) = rest.find("{#") {
            let c = col(line, offset_in(line, &rest[open..]));
            self.diag(
                i,
                c,
                format!(
                    "expected the '{{#id ...}}' block to end the heading with '}}', found {:?}",
                    &rest[open..]
                ),
            );
            rest[..open].trim_end()
        } else {
            rest
        };
        if title.is_empty() {
            self.diag(
                i,
                col(line, line.len()),
                "expected a title after the heading label, found none".into(),
            );
        }
        (title.to_string(), attrs)
    }

    fn attrs(&mut self, i: usize, inner: &str, kind: char, out: &mut Attrs) {
        let line = self.lines[i];
        for (n, tok) in inner.split_whitespace().enumerate() {
            let c = col(line, offset_in(line, tok));
            if let Some(id) = tok.strip_prefix('#') {
                if n != 0 {
                    self.diag(i, c, format!("expected the '#id' first in the braces, found {tok:?} after other tokens"));
                } else if !is_id(id, kind) {
                    self.diag(i, c, format!("expected an id like '#{kind}-7fa2' (lowercase hex, 4+ digits), found {tok:?}"));
                } else if let Some(&first) = self.ids.get(id) {
                    self.diag(
                        i,
                        c,
                        format!("duplicate id {id:?}: first used on line {first}"),
                    );
                } else {
                    self.ids.insert(id.to_string(), i + 1);
                    out.id = Some(id.to_string());
                }
            } else if kind == 'r' && tok == "protected" {
                out.protected = true;
            } else if kind == 'r'
                && let Some(list) = tok.strip_prefix("traces=")
            {
                for item in list.split(',') {
                    match item.split_once('@') {
                        Some((t, h)) if t.len() > 2 && is_id(t, t.chars().next().unwrap_or('?')) && t.chars().next().is_some_and(|c| c.is_ascii_lowercase()) && is_hex(h) => {
                            out.traces.push(Trace { target: t.to_string(), hash: h.to_string() });
                        }
                        _ => self.diag(i, c, format!("expected 'traces=<id>@<hash>' like 'traces=a-12cd@3f9e', found {tok:?}")),
                    }
                }
            } else {
                let allowed = if kind == 'r' {
                    "'#id', 'protected', 'traces=<id>@<hash>'"
                } else {
                    "'#id' only"
                };
                self.diag(i, c, format!("expected one of {allowed}, found {tok:?}"));
            }
        }
    }

    fn requirement(&mut self, i: usize, rest: &str, end: usize) -> Requirement {
        let (title, attrs) = self.title_and_attrs(i, rest, 'r');
        let text = self.lines[i + 1..end].join("\n").trim().to_string();
        Requirement {
            title,
            id: attrs.id,
            protected: attrs.protected,
            traces: attrs.traces,
            text,
            line: i + 1,
            scenarios: Vec::new(),
        }
    }

    /// A scenario's Verification line must be the first non-blank line of its
    /// block. Returns `None` after reporting when it can't be trusted.
    fn scenario(&mut self, i: usize, rest: &str, end: usize) -> Option<Scenario> {
        let (title, attrs) = self.title_and_attrs(i, rest, 's');
        let mut j = i + 1;
        while j < end && self.lines[j].trim().is_empty() {
            j += 1;
        }
        let mut sc = Scenario {
            title,
            id: attrs.id,
            line: i + 1,
            verification: Verification::NonExecutable,
            tags: Vec::new(),
            description: String::new(),
            steps: Vec::new(),
            examples: None,
        };
        if j >= end {
            self.no_marker(i, &sc.title, None);
            return Some(sc);
        }
        let mline = self.lines[j];
        if let Some(d) = self.marker_near_miss(j, mline) {
            self.diags.push(d);
            return Some(sc);
        }
        let stripped = mline.trim();
        let Some(after) = stripped.strip_prefix(MARKER_LABEL) else {
            self.no_marker(i, &sc.title, Some(mline));
            return Some(sc);
        };
        let mut toks = after.split_whitespace();
        let value = toks.next().unwrap_or("");
        sc.verification = if value == VALUE_EXECUTABLE {
            Verification::Executable
        } else {
            Verification::NonExecutable
        };
        for tok in toks {
            match tok.strip_prefix('@') {
                Some(name) if is_tag_name(name) => sc.tags.push(name.to_string()),
                _ => {
                    let c = col(mline, offset_in(mline, tok));
                    self.diag(j, c, format!("expected a tag like '@engine' after the verification value, found {tok:?}"));
                }
            }
        }
        if sc.verification == Verification::Executable {
            self.executable_body(&mut sc, j + 1, end);
        }
        Some(sc)
    }

    fn no_marker(&mut self, heading: usize, title: &str, found: Option<&str>) {
        let found = match found {
            Some(l) => format!("{:?}", l.trim()),
            None => "nothing".to_string(),
        };
        self.diag(
            heading,
            1,
            format!("expected scenario {title:?} to start with '{MARKER_LABEL} {VALUE_EXECUTABLE}' or '{MARKER_LABEL} {VALUE_NON_EXECUTABLE}', found {found}"),
        );
    }

    /// A marker line that is an attempt at the label or value but not an
    /// exact spelling; `None` for a clean marker or an unrelated line.
    fn marker_near_miss(&self, i: usize, line: &str) -> Option<Diagnostic> {
        let stripped = line.trim();
        let mk = |column: usize, message: String| Diagnostic {
            file: self.file.to_string(),
            line: i + 1,
            column,
            message,
        };
        if !stripped.starts_with(MARKER_LABEL) {
            return looks_like(line, MARKER_LABEL).then(|| {
                mk(
                    indent_col(line),
                    format!(
                        "expected the label '{MARKER_LABEL}' spelled exactly, found {stripped:?}"
                    ),
                )
            });
        }
        let after = &stripped[MARKER_LABEL.len()..];
        let Some(value) = after.split_whitespace().next() else {
            return Some(mk(
                indent_col(line),
                format!(
                    "expected '{VALUE_EXECUTABLE}' or '{VALUE_NON_EXECUTABLE}' after the label, found nothing"
                ),
            ));
        };
        if value == VALUE_EXECUTABLE || value == VALUE_NON_EXECUTABLE {
            return None;
        }
        let c = col(line, offset_in(line, value));
        Some(mk(
            c,
            format!("expected '{VALUE_EXECUTABLE}' or '{VALUE_NON_EXECUTABLE}', found {value:?}"),
        ))
    }

    fn executable_body(&mut self, sc: &mut Scenario, start: usize, end: usize) {
        let name = sc.title.clone();
        let before = self.diags.len();
        let mut description: Vec<&str> = Vec::new();
        // (keyword, fragments, line index)
        let mut steps: Vec<(Keyword, Vec<&str>, usize)> = Vec::new();
        let mut seen_examples = false;
        let mut i = start;
        while i < end {
            let raw = self.lines[i];
            let stripped = raw.trim();
            if stripped.is_empty() {
                i += 1;
                continue;
            }
            let c = indent_col(raw);
            let indented = raw.starts_with([' ', '\t']);

            if looks_like(raw, MARKER_LABEL) {
                let d = self.marker_near_miss(i, raw);
                match d {
                    Some(d) => self.diags.push(d),
                    None => self.diag(i, c, format!("expected scenario {name:?} to have one '{MARKER_LABEL}' line, found a second")),
                }
                i += 1;
                continue;
            }
            if looks_like(raw, EXAMPLES_LABEL) {
                if seen_examples {
                    self.diag(i, c, format!("expected at most one '{EXAMPLES_LABEL}' table in scenario {name:?}, found a second label"));
                    i += 1;
                } else if stripped != EXAMPLES_LABEL {
                    self.diag(i, c, format!("expected exactly '{EXAMPLES_LABEL}' on its own line, found {stripped:?}"));
                    i += 1;
                } else {
                    seen_examples = true;
                    let (ex, next) = self.examples_table(&name, i, end);
                    sc.examples = ex;
                    i = next;
                }
                continue;
            }
            if stripped.starts_with('|') && stripped.ends_with('|') {
                self.diag(i, c, format!("expected '{EXAMPLES_LABEL}' before a table in scenario {name:?}, found a table row"));
                i += 1;
                continue;
            }
            if stripped.starts_with("```") {
                self.diag(i, c, format!("expected '- **GIVEN**'/'**WHEN**'/'**THEN**' bullets in scenario {name:?}, found a code fence"));
                i += 1;
                while i < end && !self.lines[i].trim_start().starts_with("```") {
                    i += 1;
                }
                i += 1;
                continue;
            }
            if stripped.starts_with("<!--") {
                self.diag(i, c, format!("expected no HTML comment in scenario {name:?} (invisible when rendered), found {stripped:?}"));
                while i < end && !self.lines[i].contains("-->") {
                    i += 1;
                }
                i += 1;
                continue;
            }
            if !indented && stripped.starts_with('-') {
                if let Some(step) = self.step(i, &name, stripped, steps.is_empty()) {
                    steps.push(step);
                }
            } else if indented && stripped.starts_with('-') {
                self.diag(i, c, format!("expected steps at the top level in scenario {name:?}, found a nested bullet {stripped:?}"));
            } else if indented {
                match steps.last_mut() {
                    Some(s) => s.1.push(stripped),
                    None => self.diag(i, c, format!("expected a step bullet first in scenario {name:?}, found an indented line")),
                }
            } else if steps.is_empty() {
                description.push(stripped);
            } else {
                self.diag(i, c, format!("expected free text before the first step in scenario {name:?}, found {stripped:?} after steps"));
            }
            i += 1;
        }

        for (keyword, frags, line) in steps {
            let text = frags.join(" ");
            let c = indent_col(self.lines[line]);
            if text.contains("**") || text.contains('`') {
                self.diag(line, c, format!("expected plain text in scenario {name:?}'s step (matched literally by a step definition), found markup in {text:?}"));
                continue;
            }
            if let Some(ph) = bare_placeholders(&text).first() {
                let word = &ph[1..ph.len() - 1];
                self.diag(line, c, format!("expected '\\<{word}\\>' in scenario {name:?}'s step, found unescaped {ph:?}"));
                continue;
            }
            sc.steps.push(Step {
                keyword,
                text: unescape(&text),
                line: line + 1,
            });
        }
        sc.description = description.join("\n");
        self.check_scenario_shape(sc, self.diags.len() == before);
    }

    /// Checks that need the finished scenario: steps exist, and an outline's
    /// placeholders and Examples columns agree.
    fn check_scenario_shape(&mut self, sc: &Scenario, clean: bool) {
        let h = sc.line - 1;
        // Any earlier problem in the body may explain the missing steps.
        if sc.steps.is_empty() && clean {
            self.diag(
                h,
                1,
                format!(
                    "expected scenario {:?} to have at least one step, found none",
                    sc.title
                ),
            );
        }
        let Some(ex) = &sc.examples else { return };
        if ex.rows.is_empty() {
            self.diag(h, 1, format!("expected scenario outline {:?} to have an Examples data row, found only a header", sc.title));
        }
        let columns: BTreeSet<&str> = ex.header.iter().map(String::as_str).collect();
        let placeholders: BTreeSet<String> = sc
            .steps
            .iter()
            .flat_map(|s| placeholder_names(&s.text))
            .collect();
        let mut matched: BTreeSet<&str> = BTreeSet::new();
        for p in &placeholders {
            if columns.contains(p.as_str()) {
                continue;
            }
            match columns.iter().find(|c| c.eq_ignore_ascii_case(p)) {
                Some(c) => {
                    matched.insert(c);
                    self.diag(h, 1, format!("expected placeholder <{p}> to match an Examples column exactly, found column {c:?} differing only in case"));
                }
                None => self.diag(
                    h,
                    1,
                    format!("expected an Examples column for placeholder <{p}>, found none"),
                ),
            }
        }
        for c in columns {
            if !placeholders.contains(c) && !matched.contains(c) {
                self.diag(h, 1, format!("expected every Examples column to be used as a placeholder, found unused column {c:?}"));
            }
        }
    }

    fn step(
        &mut self,
        i: usize,
        name: &str,
        stripped: &str,
        first: bool,
    ) -> Option<(Keyword, Vec<&'a str>, usize)> {
        let raw = self.lines[i];
        let c = indent_col(raw);
        let body = stripped.strip_prefix('-').unwrap_or(stripped).trim_start();
        let inner = body
            .strip_prefix("**")
            .and_then(|r| r.split_once("**"))
            .filter(|(k, _)| !k.contains('*'));
        let Some((token, text)) = inner else {
            self.diag(i, c, format!("expected a bullet starting '- **GIVEN**'/'**WHEN**'/'**THEN**'/'**AND**'/'**BUT**' in scenario {name:?}, found {stripped:?}"));
            return None;
        };
        let kc = col(raw, offset_in(raw, token));
        let Some(keyword) = KEYWORDS.iter().copied().find(|k| k.as_str() == token) else {
            let hint = KEYWORDS
                .iter()
                .find(|k| k.as_str().eq_ignore_ascii_case(token))
                .map(|k| format!(" (spell it '**{}**')", k.as_str()))
                .unwrap_or_default();
            self.diag(i, kc, format!("expected a step keyword GIVEN, AND, WHEN, THEN or BUT in scenario {name:?}, found '**{token}**'{hint}"));
            return None;
        };
        if first && matches!(keyword, Keyword::And | Keyword::But) {
            self.diag(i, kc, format!("expected scenario {name:?} to start with GIVEN, WHEN or THEN, found '**{token}**'"));
            return None;
        }
        // The text slice must borrow from the file, not from `stripped`.
        let text = text.trim();
        let off = offset_in(stripped, text);
        let start = offset_in(raw, stripped) + off;
        Some((keyword, vec![&raw[start..start + text.len()]], i))
    }

    /// Parse the table after an `*Examples*:` label at line `label`.
    fn examples_table(
        &mut self,
        name: &str,
        label: usize,
        end: usize,
    ) -> (Option<Examples>, usize) {
        let mut i = label + 1;
        while i < end && self.lines[i].trim().is_empty() {
            i += 1;
        }
        let row_start = i;
        let mut raw_rows = Vec::new();
        while i < end {
            let t = self.lines[i].trim();
            if t.starts_with('|') && t.ends_with('|') {
                raw_rows.push(t);
                i += 1;
            } else {
                break;
            }
        }
        if raw_rows.is_empty() {
            self.diag(label, 1, format!("expected a markdown table under '{EXAMPLES_LABEL}' in scenario {name:?}, found none"));
            return (None, i);
        }
        let cells: Vec<Vec<String>> = raw_rows
            .iter()
            .map(|r| {
                r.trim_matches('|')
                    .split('|')
                    .map(|c| c.trim().to_string())
                    .collect()
            })
            .collect();
        let is_sep = |c: &String| {
            let t = c.trim_matches(':');
            !t.is_empty() && t.chars().all(|ch| ch == '-') && c.len() - t.len() <= 2
        };
        if cells.len() < 2 || !cells[1].iter().all(is_sep) {
            self.diag(row_start, 1, format!("expected a '| --- |' separator row under the Examples header in scenario {name:?}, found {:?}", raw_rows.get(1).unwrap_or(&"nothing")));
            return (None, i);
        }
        let header: Vec<String> = cells[0].iter().map(|c| unescape(c)).collect();
        let mut ok = true;
        for (off, row) in cells.iter().enumerate().skip(2) {
            if row.len() != header.len() {
                ok = false;
                self.diag(
                    row_start + off,
                    1,
                    format!(
                        "expected {} cell(s) in the Examples row, found {} in {:?}",
                        header.len(),
                        row.len(),
                        raw_rows[off]
                    ),
                );
            }
        }
        if !ok {
            return (None, i);
        }
        let rows = cells[2..]
            .iter()
            .map(|r| r.iter().map(|c| unescape(c)).collect())
            .collect();
        (Some(Examples { header, rows }), i)
    }
}
