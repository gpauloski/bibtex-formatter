use crate::models::{Sequence, Tag};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt::Debug;

pub trait Entry: Debug + Ord + PartialOrd {}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EntryType {
    PreambleEntry(PreambleEntry),
    StringEntry(StringEntry),
    CommentEntry(CommentEntry),
    RefEntry(RefEntry),
}

#[derive(Debug)]
pub struct Entries {
    entries: Vec<EntryType>,
    // Whitespace preceding each entry, captured verbatim from the source and
    // aligned by index with `entries`. Used only to reproduce the original
    // vertical layout when entries are not sorted. Excluded from equality since
    // it is presentation metadata; empty strings when built without spacing.
    leading: Vec<String>,
}

impl Entries {
    pub fn new(entries: Vec<EntryType>) -> Self {
        let leading = vec![String::new(); entries.len()];
        Self { entries, leading }
    }

    /// Build entries paired with the whitespace that preceded each one in the
    /// source. `leading` must be the same length as `entries`.
    pub fn with_leading(entries: Vec<EntryType>, leading: Vec<String>) -> Self {
        debug_assert_eq!(entries.len(), leading.len());
        Self { entries, leading }
    }

    pub fn iter(&self) -> impl Iterator<Item = &EntryType> {
        self.entries.iter()
    }

    /// Iterate over each entry paired with the whitespace that preceded it.
    pub fn iter_with_leading(&self) -> impl Iterator<Item = (&str, &EntryType)> {
        self.leading
            .iter()
            .map(String::as_str)
            .zip(self.entries.iter())
    }

    /// Sort entries in place by their derived ordering.
    ///
    /// NOTE: a flat sort does not preserve comment attachment (comments
    /// travelling with the entry that follows them), and it leaves the captured
    /// leading whitespace misaligned. Comment positioning and spacing are
    /// presentation concerns handled by `Formatter::format_entries`; this method
    /// is retained for API compatibility only.
    pub fn sort(&mut self) {
        self.entries.sort();
    }
}

impl PartialEq for Entries {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl Eq for Entries {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CommentKind {
    Explicit, // @comment{...}
    Implicit, // free text between entries
}

#[derive(Debug, Eq, PartialEq)]
pub struct RefEntry {
    pub kind: String,
    pub key: String,
    pub tags: Vec<Tag>,
}

impl RefEntry {
    pub const fn new(kind: String, key: String, tags: Vec<Tag>) -> Self {
        Self { kind, key, tags }
    }
}

impl Entry for RefEntry {}

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
pub struct CommentEntry {
    body: String,
    kind: CommentKind,
}

impl CommentEntry {
    pub const fn explicit(body: String) -> Self {
        Self {
            body,
            kind: CommentKind::Explicit,
        }
    }

    pub const fn implicit(body: String) -> Self {
        Self {
            body,
            kind: CommentKind::Implicit,
        }
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub const fn kind(&self) -> CommentKind {
        self.kind
    }
}

impl Entry for CommentEntry {}

impl PartialOrd for CommentEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CommentEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.body
            .cmp(&other.body)
            .then_with(|| self.kind.cmp(&other.kind))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PreambleEntry(Sequence);

impl PreambleEntry {
    pub const fn new(parts: Sequence) -> Self {
        Self(parts)
    }

    pub const fn body(&self) -> &Sequence {
        &self.0
    }
}

impl Entry for PreambleEntry {}

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

    pub const fn tag(&self) -> &Tag {
        &self.0
    }
}

impl Entry for StringEntry {}

impl PartialOrd for StringEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StringEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
