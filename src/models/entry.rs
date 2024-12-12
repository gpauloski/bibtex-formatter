use crate::models::{Sequence, Tag, Value};
use crate::Result;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Write;
use std::mem::discriminant;

pub trait Entry: Debug + Display + Ord + PartialOrd {}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EntryType {
    PreambleEntry(PreambleEntry),
    StringEntry(StringEntry),
    CommentEntry(CommentEntry),
    RefEntry(RefEntry),
}

impl Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PreambleEntry(e) => write!(f, "{e}"),
            Self::CommentEntry(e) => write!(f, "{e}"),
            Self::StringEntry(e) => write!(f, "{e}"),
            Self::RefEntry(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Entries(Vec<EntryType>);

impl Entries {
    pub fn new(entries: Vec<EntryType>) -> Self {
        Self(entries)
    }

    pub fn sort(&mut self) {
        self.0.sort();
    }

    pub fn write(&self, filepath: &str) -> Result<()> {
        let mut file = File::create(filepath)?;
        write!(file, "{self}")?;
        Ok(())
    }
}

impl Display for Entries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter().peekable();
        while let Some(entry) = iter.next() {
            if let Some(next) = iter.peek() {
                writeln!(f, "{entry}")?;

                if discriminant(entry) != discriminant(next) {
                    writeln!(f)?;
                } else if let EntryType::RefEntry(_) = next {
                    writeln!(f)?;
                }
            } else {
                write!(f, "{entry}")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RefEntry {
    pub kind: String,
    pub key: String,
    pub tags: Vec<Tag>,
}

impl RefEntry {
    pub fn new(kind: String, key: String, mut tags: Vec<Tag>) -> Self {
        tags.sort();
        Self {
            kind: kind.to_lowercase(),
            key: key.to_lowercase(),
            tags,
        }
    }
}

impl Entry for RefEntry {}

impl Display for RefEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tags.is_empty() {
            return write!(f, "@{}{{{}}}", self.kind, self.key);
        }

        writeln!(f, "@{}{{{},", self.kind, self.key)?;
        for tag in &self.tags {
            writeln!(f, "    {},", &tag)?;
        }
        write!(f, "}}")
    }
}

impl PartialOrd for RefEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RefEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommentEntry(String);

impl CommentEntry {
    pub const fn new(body: String) -> Self {
        Self(body)
    }
}

impl Entry for CommentEntry {}

impl Display for CommentEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@COMMENT{{{}}}", self.0)
    }
}

impl PartialOrd for CommentEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CommentEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PreambleEntry(Sequence);

impl PreambleEntry {
    pub const fn new(parts: Sequence) -> Self {
        Self(parts)
    }
}

impl Entry for PreambleEntry {}

impl Display for PreambleEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@PREAMBLE{{{}}}", self.0)
    }
}

impl PartialOrd for PreambleEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreambleEntry {
    fn cmp(&self, _other: &Self) -> Ordering {
        // We want to retain the order or preambles.
        Ordering::Equal
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct StringEntry(Tag);

impl StringEntry {
    pub const fn new(tag: Tag) -> Self {
        Self(tag)
    }
}

impl Entry for StringEntry {}

impl Display for StringEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match &self.0.value {
            Value::Single(s) => format!("\"{s}\""),
            Value::Integer(v) => format!("\"{v}\""),
            Value::Sequence(s) => s.to_string(),
        };
        write!(f, "@STRING{{{} = {}}}", self.0.name, value)
    }
}

impl PartialOrd for StringEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StringEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.name.cmp(&other.0.name)
    }
}
