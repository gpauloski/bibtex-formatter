use crate::models::{CommentEntry, CommentKind, Entries, EntryType, PreambleEntry, RefEntry};
use crate::models::{Part, Sequence, StringEntry, Tag, Value};
use crate::Result;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;
use std::mem::discriminant;

struct Group<'a> {
    comments: Vec<&'a EntryType>, // CommentEntry variants only
    entry: Option<&'a EntryType>, // None only for the trailing group
}

#[derive(Debug, Eq, PartialEq)]
pub struct Formatter {
    format_title: bool,
    remove_comments: bool,
    skip_empty_tags: bool,
    sort_entries: bool,
    sort_tags: bool,
}

impl Formatter {
    pub fn builder() -> FormatterBuilder {
        FormatterBuilder::default()
    }

    pub fn write_entries(&self, entries: &Entries, filepath: &str) -> Result<()> {
        let mut file = File::create(filepath)?;
        write!(file, "{}", self.format_entries(entries))?;
        Ok(())
    }

    pub fn format_entries(&self, entries: &Entries) -> String {
        // Without sorting, emit elements in their original order and reproduce
        // the source whitespace between them rather than reflowing it.
        if !self.sort_entries {
            let items: Vec<(&str, &EntryType)> = entries
                .iter_with_leading()
                .filter(|(_, e)| !(self.remove_comments && matches!(e, EntryType::CommentEntry(_))))
                .collect();
            return self.format_in_order(&items);
        }

        let entries: Vec<&EntryType> = entries
            .iter()
            .filter(|e| !(self.remove_comments && matches!(e, EntryType::CommentEntry(_))))
            .collect();

        // Attach each run of comments to the next non-comment entry; leftovers
        // form a trailing entry-less group that must stay last.
        let mut groups: Vec<Group> = Vec::new();
        let mut pending: Vec<&EntryType> = Vec::new();
        for entry in entries {
            if matches!(entry, EntryType::CommentEntry(_)) {
                pending.push(entry);
            } else {
                groups.push(Group {
                    comments: std::mem::take(&mut pending),
                    entry: Some(entry),
                });
            }
        }
        if !pending.is_empty() {
            groups.push(Group {
                comments: pending,
                entry: None,
            });
        }

        groups.sort_by(|a, b| match (a.entry, b.entry) {
            (Some(x), Some(y)) => x.cmp(y), // derived EntryType Ord, unchanged
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        });

        let mut out = String::new();
        for (i, group) in groups.iter().enumerate() {
            if i > 0 {
                out.push('\n');
                if Self::blank_line_between(&groups[i - 1], group) {
                    out.push('\n');
                }
            }
            let parts: Vec<String> = group
                .comments
                .iter()
                .chain(group.entry.iter())
                .map(|e| self.format_entry(e))
                .collect();
            out.push_str(&parts.join("\n")); // comments flush above their entry
        }
        out
    }

    /// Emit entries in their original order, reproducing the source whitespace
    /// that preceded each element so nothing is reflowed when sorting is
    /// disabled.
    fn format_in_order(&self, items: &[(&str, &EntryType)]) -> String {
        let mut out = String::new();
        for (i, (leading, entry)) in items.iter().enumerate() {
            if i > 0 {
                out.push_str(&Self::separator_before(leading, items[i - 1].1, entry));
            }
            out.push_str(&self.format_entry(entry));
        }
        out
    }

    /// Choose the whitespace separating two adjacent elements when preserving
    /// order. The captured source whitespace takes precedence; when it is absent
    /// (e.g. hand-built entries) fall back to the same rules the sorted path
    /// uses.
    fn separator_before(leading: &str, prev: &EntryType, cur: &EntryType) -> String {
        if !leading.is_empty() {
            return newlines(leading);
        }
        // No captured whitespace: a comment block is set off by a blank line and
        // sits flush above its following entry, matching the sorted path.
        if matches!(cur, EntryType::CommentEntry(_)) {
            "\n\n".to_string()
        } else if matches!(prev, EntryType::CommentEntry(_)) {
            "\n".to_string()
        } else if discriminant(prev) != discriminant(cur) || matches!(cur, EntryType::RefEntry(_)) {
            "\n\n".to_string()
        } else {
            "\n".to_string()
        }
    }

    fn blank_line_between(prev: &Group, next: &Group) -> bool {
        if !next.comments.is_empty() {
            return true; // a comment block is always set off by a blank line
        }
        match (prev.entry, next.entry) {
            // Legacy rule, verbatim: different variants, or next is a RefEntry.
            (Some(a), Some(b)) => {
                discriminant(a) != discriminant(b) || matches!(b, EntryType::RefEntry(_))
            }
            _ => true,
        }
    }

    pub fn format_entry(&self, entry: &EntryType) -> String {
        match entry {
            EntryType::CommentEntry(e) => self.format_comment_entry(e),
            EntryType::PreambleEntry(e) => self.format_preamble_entry(e),
            EntryType::RefEntry(e) => self.format_ref_entry(e),
            EntryType::StringEntry(e) => self.format_string_entry(e),
        }
    }

    pub fn format_comment_entry(&self, entry: &CommentEntry) -> String {
        match entry.kind() {
            CommentKind::Explicit => format!("@COMMENT{{{}}}", entry.body()),
            CommentKind::Implicit => entry.body().to_string(),
        }
    }

    pub fn format_preamble_entry(&self, entry: &PreambleEntry) -> String {
        format!(
            "@PREAMBLE{{{}}}",
            self.format_value_sequence("preamble", entry.body())
        )
    }

    pub fn format_ref_entry(&self, entry: &RefEntry) -> String {
        let mut tags: Vec<&Tag> = if self.skip_empty_tags {
            entry
                .tags
                .iter()
                .filter(|tag| !tag.value.is_empty())
                .collect()
        } else {
            entry.tags.iter().collect()
        };

        if tags.is_empty() {
            return format!(
                "@{}{{{}}}",
                entry.kind.to_lowercase(),
                entry.key.to_lowercase()
            );
        }

        if self.sort_tags {
            tags.sort();
        }

        let mut formatted = String::new();
        formatted.push_str(&format!(
            "@{}{{{},\n",
            entry.kind.to_lowercase(),
            entry.key.to_lowercase()
        ));
        for tag in &tags {
            formatted.push_str(&format!("    {},\n", self.format_tag(tag)));
        }
        formatted.push('}');
        formatted
    }

    pub fn format_string_entry(&self, entry: &StringEntry) -> String {
        let tag = entry.tag();
        let value = match &tag.value {
            Value::Single(s) => format!("\"{s}\""),
            Value::Integer(v) => format!("\"{v}\""),
            Value::Sequence(s) => self.format_value_sequence(&tag.name, s),
        };
        format!("@STRING{{{} = {}}}", tag.name.to_lowercase(), value)
    }

    pub fn format_tag(&self, tag: &Tag) -> String {
        let name = tag.name.to_lowercase();
        let value = self.format_value(&name, &tag.value);
        format!("{} = {}", name, value)
    }

    fn format_value(&self, name: &str, value: &Value) -> String {
        match value {
            Value::Single(s) => {
                if self.preserve_tag_casing(name) {
                    format!("{{{}}}", format_title(s))
                } else {
                    format!("{{{s}}}")
                }
            }
            Value::Integer(s) => format!("{s}"),
            Value::Sequence(s) => self.format_value_sequence(name, s),
        }
    }

    fn format_value_sequence(&self, name: &str, seq: &Sequence) -> String {
        seq.parts()
            .iter()
            .map(|part| self.format_value_part(name, part))
            .collect::<Vec<String>>()
            .join(" # ")
    }

    fn format_value_part(&self, name: &str, part: &Part) -> String {
        match part {
            Part::Quoted(s) => {
                if self.preserve_tag_casing(name) {
                    format!("\"{}\"", format_title(s))
                } else {
                    format!("\"{s}\"")
                }
            }
            Part::Value(v) => v.to_lowercase(),
        }
    }

    fn preserve_tag_casing(&self, name: &str) -> bool {
        self.format_title && name == "title"
    }
}

pub struct FormatterBuilder {
    format_title: bool,
    remove_comments: bool,
    skip_empty_tags: bool,
    sort_entries: bool,
    sort_tags: bool,
}

impl Default for FormatterBuilder {
    fn default() -> Self {
        Self {
            format_title: true,
            remove_comments: false,
            skip_empty_tags: true,
            sort_entries: true,
            sort_tags: true,
        }
    }
}

impl FormatterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn build(self) -> Formatter {
        Formatter {
            format_title: self.format_title,
            remove_comments: self.remove_comments,
            skip_empty_tags: self.skip_empty_tags,
            sort_entries: self.sort_entries,
            sort_tags: self.sort_tags,
        }
    }

    pub const fn format_title(mut self, format_title: bool) -> Self {
        self.format_title = format_title;
        self
    }

    pub const fn remove_comments(mut self, remove_comments: bool) -> Self {
        self.remove_comments = remove_comments;
        self
    }

    pub const fn skip_empty_tags(mut self, skip_empty_tags: bool) -> Self {
        self.skip_empty_tags = skip_empty_tags;
        self
    }

    pub const fn sort_entries(mut self, sort_entries: bool) -> Self {
        self.sort_entries = sort_entries;
        self
    }

    pub const fn sort_tags(mut self, sort_tags: bool) -> Self {
        self.sort_tags = sort_tags;
        self
    }
}

pub fn remove_braces(text: &str) -> String {
    text.replace(&['{', '}'][..], "")
}

/// Reproduce the newline structure of a captured whitespace run, dropping any
/// spaces/indentation but keeping blank lines. Always emits at least one newline
/// since adjacent elements are on separate lines.
fn newlines(ws: &str) -> String {
    "\n".repeat(ws.matches('\n').count().max(1))
}

fn split_with_delimiters(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    for c in input.chars() {
        if c.is_whitespace() || matches!(c, '-' | '–' | '—' | '.' | ':' | '!' | '(' | ')') {
            if !current.is_empty() {
                result.push(current.clone());
                current.clear();
            }
            // Add delimiter as it's own element.
            result.push(c.to_string());
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        // Add the last segment if any.
        result.push(current);
    }

    result
}

fn wrap_first_char_with_braces(word: &str) -> String {
    if let Some((first, rest)) = word.split_at(1).into() {
        format!("{{{}}}{}", first, rest)
    } else {
        word.to_string()
    }
}

fn wrap_word_with_braces(word: &str) -> String {
    word.strip_suffix(':').map_or_else(
        || format!("{{{}}}", word),
        |stripped| format!("{{{}}}:", stripped),
    )
}

pub fn format_title(text: &str) -> String {
    let normalized = remove_braces(text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    split_with_delimiters(&normalized)
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let mut chars = word.chars();
            let first_cap = chars.next().map_or_else(|| false, |c| c.is_uppercase());
            let rest_cap = chars.any(|c| c.is_uppercase());

            if first_cap && !rest_cap {
                // Bibtex automatically capitalizes first char so first word does
                // not need to be wrapped if only its first char is a capital.
                if i == 0 {
                    word.to_string()
                } else {
                    wrap_first_char_with_braces(word)
                }
            } else if rest_cap {
                // Wrap entire word if any char other than the first is a capital.
                wrap_word_with_braces(word)
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CommentEntry, Entries, RefEntry};
    use test_case::test_case;

    fn ref_entry(key: &str) -> EntryType {
        EntryType::RefEntry(RefEntry::new(
            "misc".to_string(),
            key.to_string(),
            Vec::with_capacity(0),
        ))
    }

    #[test]
    fn test_format_comment_entry_explicit() {
        let formatter = Formatter::builder().build();
        let entry = CommentEntry::explicit(" body ".to_string());
        assert_eq!(formatter.format_comment_entry(&entry), "@COMMENT{ body }");
    }

    #[test]
    fn test_format_comment_entry_implicit() {
        let formatter = Formatter::builder().build();
        let entry = CommentEntry::implicit("free text".to_string());
        assert_eq!(formatter.format_comment_entry(&entry), "free text");
    }

    #[test]
    fn test_format_entries_comment_attachment() {
        let formatter = Formatter::builder().build();
        let entries = Entries::new(vec![
            EntryType::CommentEntry(CommentEntry::implicit("a note".to_string())),
            ref_entry("z"),
            ref_entry("a"),
        ]);
        let expected = "@misc{a}\n\na note\n@misc{z}";
        assert_eq!(formatter.format_entries(&entries), expected);
    }

    #[test]
    fn test_format_entries_trailing_comments_stay_last() {
        let formatter = Formatter::builder().build();
        let entries = Entries::new(vec![
            ref_entry("z"),
            EntryType::CommentEntry(CommentEntry::implicit("trailing".to_string())),
            ref_entry("a"),
            EntryType::CommentEntry(CommentEntry::implicit("last".to_string())),
        ]);
        // "trailing" attaches to the following entry a; "last" has no following
        // entry so it stays in a trailing group after every sorted entry.
        let expected = "trailing\n@misc{a}\n\n@misc{z}\n\nlast";
        assert_eq!(formatter.format_entries(&entries), expected);
    }

    #[test]
    fn test_format_entries_remove_comments() {
        let formatter = Formatter::builder().remove_comments(true).build();
        let entries = Entries::new(vec![
            EntryType::CommentEntry(CommentEntry::implicit("a note".to_string())),
            ref_entry("z"),
            ref_entry("a"),
        ]);
        assert_eq!(formatter.format_entries(&entries), "@misc{a}\n\n@misc{z}");

        let comments_only = Entries::new(vec![EntryType::CommentEntry(CommentEntry::implicit(
            "note".to_string(),
        ))]);
        assert_eq!(formatter.format_entries(&comments_only), "");
    }

    #[test]
    fn test_format_entries_skip_sort_with_comments() {
        let formatter = Formatter::builder().sort_entries(false).build();
        let entries = Entries::new(vec![
            EntryType::CommentEntry(CommentEntry::implicit("a note".to_string())),
            ref_entry("z"),
            ref_entry("a"),
        ]);
        let expected = "a note\n@misc{z}\n\n@misc{a}";
        assert_eq!(formatter.format_entries(&entries), expected);
    }

    #[test]
    fn test_format_entries_skip_sort_preserves_comment_whitespace() {
        let formatter = Formatter::builder().sort_entries(false).build();
        let entries = Entries::with_leading(
            vec![
                ref_entry("a"),
                EntryType::CommentEntry(CommentEntry::implicit("note".to_string())),
                ref_entry("b"),
            ],
            vec![String::new(), "\n\n\n".to_string(), "\n\n".to_string()],
        );
        // Blank lines around the comment are reproduced verbatim: two blank
        // lines before it, one blank line after it.
        let expected = "@misc{a}\n\n\nnote\n\n@misc{b}";
        assert_eq!(formatter.format_entries(&entries), expected);
    }

    #[test]
    fn test_format_entries_skip_sort_preserves_entry_whitespace() {
        let formatter = Formatter::builder().sort_entries(false).build();
        let entries = Entries::with_leading(
            vec![ref_entry("z"), ref_entry("y"), ref_entry("x")],
            vec![String::new(), "\n".to_string(), "\n\n\n".to_string()],
        );
        // Comment-free spacing is preserved verbatim: z and y flush (single
        // newline), two blank lines before x.
        let expected = "@misc{z}\n@misc{y}\n\n\n@misc{x}";
        assert_eq!(formatter.format_entries(&entries), expected);
    }

    #[test]
    fn test_format_entries_skip_sort_preserves_explicit_comment_whitespace() {
        let formatter = Formatter::builder().sort_entries(false).build();
        let entries = Entries::with_leading(
            vec![
                EntryType::CommentEntry(CommentEntry::explicit("note".to_string())),
                ref_entry("a"),
            ],
            vec![String::new(), "\n\n\n".to_string()],
        );
        // The two blank lines after the @comment are reproduced verbatim.
        let expected = "@COMMENT{note}\n\n\n@misc{a}";
        assert_eq!(formatter.format_entries(&entries), expected);
    }

    #[test]
    fn test_format_entries_comment_before_first_entry_travels() {
        let formatter = Formatter::builder().build();
        let entries = Entries::new(vec![
            EntryType::CommentEntry(CommentEntry::implicit("header".to_string())),
            ref_entry("z"),
            ref_entry("a"),
        ]);
        // The header comment attaches to z and travels with it under sort.
        let expected = "@misc{a}\n\nheader\n@misc{z}";
        assert_eq!(formatter.format_entries(&entries), expected);
    }

    #[test]
    fn test_formatter_builder() {
        let formatter = Formatter {
            format_title: true,
            remove_comments: false,
            skip_empty_tags: false,
            sort_entries: true,
            sort_tags: false,
        };
        let formatter_from_builder = FormatterBuilder::new()
            .skip_empty_tags(false)
            .sort_tags(false)
            .build();
        assert_eq!(formatter, formatter_from_builder)
    }

    #[test_case("foo", "foo" ; "default")]
    #[test_case("{foo}", "foo" ; "simple")]
    #[test_case("{foo} {} {bar}}", "foo  bar" ; "braces complex")]
    fn test_remove_braces(input: &str, expected: &str) {
        assert_eq!(remove_braces(input), expected)
    }

    #[test_case("foo", "{foo}" ; "default")]
    #[test_case("foo:", "{foo}:" ; "exclude colon")]
    fn test_wrap_word_with_braces(input: &str, expected: &str) {
        assert_eq!(wrap_word_with_braces(input), expected)
    }

    #[test_case("foo", "foo" ; "default")]
    #[test_case("{foo}", "foo" ; "simple")]
    #[test_case("Foo {FOO}", "Foo {FOO}" ; "skip first character")]
    #[test_case("FOO:", "{FOO}:" ; "exclude colon")]
    #[test_case("Foo-Bar-BAZ", "Foo-{B}ar-{BAZ}" ; "split dashes")]
    #[test_case("{FOO: A Framework for BaR}", "{FOO}: {A} {F}ramework for {BaR}" ; "multiple")]
    fn test_format_title(input: &str, expected: &str) {
        assert_eq!(format_title(input), expected)
    }
}
