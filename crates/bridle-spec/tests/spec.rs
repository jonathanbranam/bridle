#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bridle_spec::{Keyword, Verification, parse_file, parse_str};

fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut v: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path().join("spec.md"))
        .collect();
    v.sort();
    v
}

fn errs(text: &str) -> Vec<String> {
    parse_str("t.md", text)
        .expect_err("expected diagnostics")
        .iter()
        .map(|d| d.to_string())
        .collect()
}

/// Exactly one diagnostic, at `line:col`, containing `needle`.
fn one(text: &str, line: usize, col: usize, needle: &str) {
    let e = errs(text);
    assert_eq!(e.len(), 1, "{e:#?}");
    assert!(
        e[0].starts_with(&format!("t.md:{line}:{col}: ")),
        "{}",
        e[0]
    );
    assert!(e[0].contains(needle), "{} lacks {needle:?}", e[0]);
}

const GOOD: &str = "# Engine\n\n## Requirements\n\n### Requirement: Referee   {#r-7fa2 protected traces=a-12cd@3f9e}\n\nThe engine SHALL referee.\n\n#### Scenario: Reach   {#s-b310}\n\n*Verification*: **executable** @engine @polars\n\n- **GIVEN** a unit\n  with reach 2\n- **WHEN** asked for \\<x\\>\n- **THEN** ok\n";

fn wrap(scenario_body: &str) -> String {
    format!("### Requirement: R\n\nSHALL.\n\n#### Scenario: S\n\n{scenario_body}")
}

#[test]
fn ids_and_flags_round_trip() {
    let spec = parse_str("t.md", GOOD).unwrap();
    assert_eq!(spec.title.as_deref(), Some("Engine"));
    let r = &spec.requirements[0];
    assert_eq!(r.title, "Referee");
    assert_eq!(r.id.as_deref(), Some("r-7fa2"));
    assert!(r.protected);
    assert_eq!(
        (r.traces[0].target.as_str(), r.traces[0].hash.as_str()),
        ("a-12cd", "3f9e")
    );
    assert_eq!(r.text, "The engine SHALL referee.");
    assert_eq!(r.line, 5);
    let s = &r.scenarios[0];
    assert_eq!(s.title, "Reach");
    assert_eq!(s.id.as_deref(), Some("s-b310"));
    assert_eq!(s.verification, Verification::Executable);
    assert_eq!(s.tags, ["engine", "polars"]);
    assert_eq!(s.steps[0].keyword, Keyword::Given);
    assert_eq!(s.steps[0].text, "a unit with reach 2");
    assert_eq!(s.steps[1].text, "asked for <x>");
    assert_eq!(s.steps[2].keyword, Keyword::Then);
}

#[test]
fn ids_are_optional_and_non_executable_body_is_not_parsed() {
    let spec = parse_str(
        "t.md",
        &wrap("*Verification*: **non-executable**\n\n- anything ```goes\n"),
    )
    .unwrap();
    let s = &spec.requirements[0].scenarios[0];
    assert_eq!(spec.requirements[0].id, None);
    assert_eq!(s.verification, Verification::NonExecutable);
    assert!(s.steps.is_empty());
}

#[test]
fn examples_table_parses() {
    let body = "*Verification*: **executable**\n\n- **GIVEN** a \\<v\\>\n\n*Examples*:\n\n| v |\n| --- |\n| 1 |\n| 2 |\n";
    let spec = parse_str("t.md", &wrap(body)).unwrap();
    let ex = spec.requirements[0].scenarios[0].examples.as_ref().unwrap();
    assert_eq!(ex.header, ["v"]);
    assert_eq!(ex.rows, [["1"], ["2"]]);
}

#[test]
fn wrong_heading_level() {
    one("## Requirement: R\n", 1, 1, "### Requirement:");
    one(
        "### Requirement: R\n\n### Scenario: S\n\n#### Scenario: T\n*Verification*: **non-executable**\n",
        3,
        1,
        "#### Scenario:",
    );
    one("###Requirement: R\n", 1, 1, "### Requirement:");
    one(
        "### Requirement: R\n\n#### Scenarios\n\n#### Scenario: T\n*Verification*: **non-executable**\n",
        3,
        1,
        "#### Scenario:",
    );
}

#[test]
fn empty_titles_and_orphan_scenario() {
    let e = errs("### Requirement:\n\n#### Scenario: \n*Verification*: **non-executable**\n");
    assert!(
        e.iter()
            .any(|m| m.starts_with("t.md:1:") && m.contains("title")),
        "{e:?}"
    );
    assert!(
        e.iter()
            .any(|m| m.starts_with("t.md:3:") && m.contains("title")),
        "{e:?}"
    );
    one(
        "#### Scenario: S\n\n*Verification*: **non-executable**\n",
        1,
        1,
        "before any requirement",
    );
}

#[test]
fn missing_and_misspelt_verification() {
    one(&wrap("- **GIVEN** x\n"), 5, 1, "found \"- **GIVEN** x\"");
    one(
        "### Requirement: R\n\n#### Scenario: S\n",
        3,
        1,
        "found nothing",
    );
    one(
        &wrap("Verification: **executable**\n"),
        7,
        1,
        "spelled exactly",
    );
    one(
        &wrap("*verification*: **executable**\n"),
        7,
        1,
        "spelled exactly",
    );
    one(
        &wrap("*Verification*: **Executable**\n"),
        7,
        17,
        "found \"**Executable**\"",
    );
    one(&wrap("*Verification*: manual\n"), 7, 17, "found \"manual\"");
    one(&wrap("*Verification*:\n"), 7, 1, "found nothing");
    one(
        &wrap("*Verification*: **executable** engine\n\n- **GIVEN** a\n"),
        7,
        32,
        "tag",
    );
}

#[test]
fn second_marker() {
    one(
        &wrap("*Verification*: **executable**\n*Verification*: **executable**\n\n- **GIVEN** a\n"),
        8,
        1,
        "second",
    );
}

#[test]
fn unknown_and_misspelt_step_keywords() {
    one(
        &wrap("*Verification*: **executable**\n\n- **SUPPOSE** a\n"),
        9,
        5,
        "found '**SUPPOSE**'",
    );
    one(
        &wrap("*Verification*: **executable**\n\n- **Given** a\n"),
        9,
        5,
        "spell it '**GIVEN**'",
    );
    one(
        &wrap("*Verification*: **executable**\n\n- a plain bullet\n"),
        9,
        1,
        "found \"- a plain bullet\"",
    );
    one(
        &wrap("*Verification*: **executable**\n\n- **AND** a\n"),
        9,
        5,
        "start with GIVEN",
    );
}

#[test]
fn executable_body_rejections() {
    let m = "*Verification*: **executable**\n\n";
    one(&wrap(&format!("{m}- **GIVEN** a `code`\n")), 9, 1, "markup");
    one(
        &wrap(&format!("{m}- **GIVEN** a <thing>\n")),
        9,
        1,
        "\\<thing\\>",
    );
    one(
        &wrap(&format!("{m}- **GIVEN** a\n\n```\nx\n```\n")),
        11,
        1,
        "code fence",
    );
    one(
        &wrap(&format!("{m}- **GIVEN** a\n<!-- c -->\n")),
        10,
        1,
        "HTML comment",
    );
    one(
        &wrap(&format!("{m}- **GIVEN** a\n  - nested\n")),
        10,
        3,
        "nested",
    );
    one(
        &wrap(&format!("{m}- **GIVEN** a\n\nfree text\n")),
        11,
        1,
        "after steps",
    );
    one(
        &wrap(&format!("{m}  indented\n- **GIVEN** a\n")),
        9,
        3,
        "indented",
    );
    one(
        &wrap(&format!("{m}| a |\n- **GIVEN** a\n")),
        9,
        1,
        "Examples",
    );
    one(&wrap(m), 5, 1, "at least one step");
}

#[test]
fn examples_rejections() {
    let m = "*Verification*: **executable**\n\n- **GIVEN** a \\<v\\>\n\n";
    assert!(errs(&wrap(&format!("{m}*Examples*:\n\nnope\n")))[0].contains("table"));
    one(
        &wrap(&format!("{m}*Examples*:\n\n| v |\n| 1 |\n")),
        13,
        1,
        "separator",
    );
    one(
        &wrap(&format!("{m}*Examples*:\n\n| v |\n| - |\n| 1 | 2 |\n")),
        15,
        1,
        "cell(s)",
    );
    one(
        &wrap(&format!("{m}*Examples*:\n\n| v |\n| - |\n")),
        5,
        1,
        "data row",
    );
    one(
        &wrap(&format!(
            "{m}*Examples*:\n\n| v | w |\n| - | - |\n| 1 | 2 |\n"
        )),
        5,
        1,
        "unused column",
    );
    one(
        &wrap(&format!("{m}*Examples*:\n\n| V |\n| - |\n| 1 |\n")),
        5,
        1,
        "case",
    );
    one(&wrap(&format!("{m}*Examples*: x\n")), 11, 1, "own line");
}

#[test]
fn duplicate_ids() {
    let t = "### Requirement: A {#r-1111}\n\n#### Scenario: S {#s-2222}\n*Verification*: **non-executable**\n\n### Requirement: B {#r-1111}\n\n#### Scenario: T {#s-2222}\n*Verification*: **non-executable**\n";
    let e = errs(t);
    assert_eq!(e.len(), 2, "{e:?}");
    assert!(
        e[0].starts_with("t.md:6:21:") && e[0].contains("first used on line 1"),
        "{}",
        e[0]
    );
    assert!(e[1].starts_with("t.md:8:19:"), "{}", e[1]);
}

#[test]
fn bad_attribute_blocks() {
    let sc = "\n#### Scenario: S\n*Verification*: **non-executable**\n";
    one(
        &format!("### Requirement: R {{#x-12}}\n{sc}"),
        1,
        21,
        "expected an id",
    );
    one(
        &format!("### Requirement: R {{#s-1234}}\n{sc}"),
        1,
        21,
        "expected an id",
    );
    one(
        &format!("### Requirement: R {{shiny}}\n{sc}"),
        1,
        21,
        "\"shiny\"",
    );
    one(
        &format!("### Requirement: R {{protected #r-1234}}\n{sc}"),
        1,
        31,
        "first",
    );
    one(
        &format!("### Requirement: R {{traces=a-12cd}}\n{sc}"),
        1,
        21,
        "traces=<id>@<hash>",
    );
    one(
        &format!("### Requirement: R {{#r-1234\n{sc}"),
        1,
        20,
        "end the heading",
    );
    one(
        "### Requirement: R\n\n#### Scenario: S {protected}\n*Verification*: **non-executable**\n",
        3,
        19,
        "'#id' only",
    );
}

#[test]
fn requirement_without_scenario() {
    one(
        "### Requirement: R\n\nSHALL.\n",
        1,
        1,
        "at least one '#### Scenario:'",
    );
}

#[test]
fn reports_every_diagnostic() {
    let e = errs(
        "## Requirement: A\n\n### Requirement: B\n\n#### Scenario: S\n\n#### Scenario: T\n*Verification*: nope\n",
    );
    assert_eq!(e.len(), 3, "{e:#?}");
    let lines: Vec<_> = e
        .iter()
        .map(|m| m.split(':').nth(1).unwrap().to_string())
        .collect();
    assert_eq!(lines, ["1", "5", "8"]);
}

#[test]
fn fixtures_parse() {
    let paths = fixtures();
    assert_eq!(paths.len(), 6);
    let mut total = (0, 0, 0);
    for p in &paths {
        let spec = parse_file(p).unwrap_or_else(|e| panic!("{}: {e:?}", p.display()));
        total.0 += spec.requirements.len();
        for r in &spec.requirements {
            assert!(r.text.contains("SHALL"), "{}: {}", p.display(), r.title);
            total.1 += r.scenarios.len();
            total.2 += r
                .scenarios
                .iter()
                .filter(|s| s.verification == Verification::Executable)
                .count();
        }
    }
    assert_eq!(total, (56, 86, 76));
}

#[test]
fn fixture_with_examples_outline() {
    let spec = parse_file(
        &fixtures()
            .iter()
            .find(|p| p.to_string_lossy().contains("/verification/"))
            .unwrap()
            .clone(),
    )
    .unwrap();
    let outline = spec
        .requirements
        .iter()
        .flat_map(|r| &r.scenarios)
        .find(|s| s.examples.is_some())
        .unwrap();
    assert!(!outline.examples.as_ref().unwrap().rows.is_empty());
}

#[test]
#[ignore = "wall-clock benchmark; run with --ignored"]
fn parses_fast() {
    // Pathological-slowdown guard: verifies that parsing every fixture 20 times
    // doesn't degrade to quadratic or worse behavior. Wall-clock bounds are too
    // load-sensitive for just check; run with `cargo nextest run -p bridle-spec --run-ignored only parses_fast`.
    let start = Instant::now();
    for _ in 0..20 {
        for p in fixtures() {
            parse_file(&p).unwrap();
        }
    }
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "{:?}",
        start.elapsed()
    );
}

#[test]
fn parse_file_reports_io_and_diagnostics() {
    assert!(matches!(
        parse_file(Path::new("/nonexistent/x.md")),
        Err(bridle_spec::ParseFileError::Io { .. })
    ));
}
