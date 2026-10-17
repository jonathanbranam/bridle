//! `@role` mentions in a comment (br-4cgx): who a comment names.

/// The distinct names `text` mentions, in order of first appearance. An `@` counts only at the
/// start of the text or after a character that cannot end a name (so `a@b` and email addresses
/// are not mentions), and `\@name` is escaped plain text. A name is letters, digits, `-`, `_`,
/// `/` (named sessions: `advisor/product-manager`) and `:` (a project qualifier, not resolved
/// here); a trailing `/` or `:` (sentence punctuation) is dropped.
pub fn mentioned(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<String> = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if c != '@' {
            continue;
        }
        if i > 0 {
            let prev = chars[i - 1];
            if prev == '\\' || prev.is_alphanumeric() || matches!(prev, '_' | '.' | '-' | '/') {
                continue;
            }
        }
        let name: String = chars[i + 1..]
            .iter()
            .take_while(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '/' | ':'))
            .collect();
        let name = name.trim_end_matches(['/', ':']);
        if !name.is_empty() && !out.iter().any(|n| n == name) {
            out.push(name.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::mentioned;

    #[test]
    fn finds_each_distinct_name_once() {
        assert_eq!(
            mentioned("@orchestrator I approve. cc @advisor/product-manager, @orchestrator."),
            ["orchestrator", "advisor/product-manager"]
        );
    }

    #[test]
    fn a_backslash_escapes_the_mention() {
        assert!(mentioned(r"\@orchestrator is plain text").is_empty());
    }

    #[test]
    fn an_email_address_is_not_a_mention() {
        assert!(mentioned("mail dev@example.com or a@b").is_empty());
    }

    #[test]
    fn a_trailing_colon_is_punctuation_and_a_project_qualifier_stays() {
        assert_eq!(mentioned("@aide: look"), ["aide"]);
        assert_eq!(mentioned("(@ui:orchestrator)"), ["ui:orchestrator"]);
    }
}
