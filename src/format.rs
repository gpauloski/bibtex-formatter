use crate::models::{CommentEntry, Entries, EntryType, PreambleEntry, RefEntry, StringEntry};
use crate::models::{Part, Sequence, Tag, Value};
use crate::Result;
use std::fs::File;
use std::io::Write;
use std::mem::discriminant;

#[derive(Debug, Eq, PartialEq)]
pub struct Formatter {
    format_title: bool,
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
        let mut lines: Vec<String> = vec![];
        let mut entries: Vec<&EntryType> = entries.iter().collect();
        if self.sort_entries {
            entries.sort();
        }
        let mut iter = entries.iter().peekable();

        while let Some(entry) = iter.next() {
            if let Some(next) = iter.peek() {
                lines.push(format!("{}\n", self.format_entry(entry)));

                if discriminant(*entry) != discriminant(*next) {
                    lines.push("\n".to_string());
                } else if let EntryType::RefEntry(_) = next {
                    lines.push("\n".to_string());
                }
            } else {
                lines.push(self.format_entry(entry));
            }
        }

        lines.join("")
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
        format!("@COMMENT{{{}}}", entry.body())
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
    skip_empty_tags: bool,
    sort_entries: bool,
    sort_tags: bool,
}

impl Default for FormatterBuilder {
    fn default() -> Self {
        Self {
            format_title: true,
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
            skip_empty_tags: self.skip_empty_tags,
            sort_entries: self.sort_entries,
            sort_tags: self.sort_tags,
        }
    }

    pub const fn format_title(mut self, format_title: bool) -> Self {
        self.format_title = format_title;
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
    remove_braces(text)
        .split_whitespace()
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
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_formatter_builder() {
        let formatter = Formatter {
            format_title: true,
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
    #[test_case("{FOO: A Framework for BaR}", "{FOO}: {A} {F}ramework for {BaR}" ; "multiple")]
    fn test_format_title(input: &str, expected: &str) {
        assert_eq!(format_title(input), expected)
    }
}
