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

#[derive(Debug, Eq, PartialEq)]
pub struct Entries(Vec<EntryType>);

impl Entries {
    pub fn new(entries: Vec<EntryType>) -> Self {
        Self(entries)
    }

    pub fn iter(&self) -> impl Iterator<Item = &EntryType> {
        self.0.iter()
    }

    pub fn sort(&mut self) {
        self.0.sort();
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RefEntry {
    pub kind: String,
    pub key: String,
    pub tags: Vec<Tag>,
}

impl RefEntry {
    pub fn new(kind: String, key: String, tags: Vec<Tag>) -> Self {
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
pub struct CommentEntry(String);

impl CommentEntry {
    pub const fn new(body: String) -> Self {
        Self(body)
    }

    pub fn body(&self) -> &str {
        &self.0
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
        self.0.cmp(&other.0)
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
