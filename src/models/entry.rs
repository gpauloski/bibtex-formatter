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

    /// Remove exact-duplicate reference entries in place, keeping the first
    /// occurrence of each. Two reference entries are exact duplicates when they
    /// share a cite key and format identically (see [`RefEntry::duplicates`]);
    /// collapsing them is lossless.
    ///
    /// Reference entries that share a cite key but differ in content are *not*
    /// removed — dropping one would silently change what a document cites.
    /// Instead every such collision is reported in the returned warnings, and
    /// all conflicting entries are kept. Non-reference entries (comments,
    /// strings, preambles) are never touched.
    ///
    /// Detection is independent of sorting, so this composes with both the
    /// sorted and order-preserving formatting paths.
    pub fn remove_duplicates(&mut self) -> Vec<String> {
        let mut keep = vec![true; self.entries.len()];
        // Kept reference-entry indices grouped by lowercased cite key, in
        // first-seen order so warnings are deterministic.
        let mut by_key: Vec<(String, Vec<usize>)> = Vec::new();

        for (i, entry) in self.entries.iter().enumerate() {
            let EntryType::RefEntry(current) = entry else {
                continue;
            };
            let key = current.key.to_lowercase();
            let group = match by_key.iter().position(|(k, _)| *k == key) {
                Some(pos) => &mut by_key[pos].1,
                None => {
                    by_key.push((key, Vec::new()));
                    &mut by_key.last_mut().unwrap().1
                }
            };
            let is_duplicate = group.iter().any(|&j| {
                let EntryType::RefEntry(kept) = &self.entries[j] else {
                    unreachable!("group only holds reference-entry indices")
                };
                kept.duplicates(current)
            });
            if is_duplicate {
                keep[i] = false;
            } else {
                group.push(i);
            }
        }

        let warnings = by_key
            .iter()
            .filter(|(_, group)| group.len() > 1)
            .map(|(key, group)| {
                format!(
                    "warning: cite key '{}' has {} conflicting definitions; keeping all",
                    key,
                    group.len()
                )
            })
            .collect();

        let mut index = 0;
        self.leading.retain(|_| {
            let kept = keep[index];
            index += 1;
            kept
        });
        let mut index = 0;
        self.entries.retain(|_| {
            let kept = keep[index];
            index += 1;
            kept
        });

        warnings
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

    /// Whether two reference entries are exact duplicates: same kind and cite
    /// key (compared case-insensitively, as both are lowercased on output) and
    /// the same set of tags regardless of order (tags are sorted on output).
    /// Two such entries format identically, so one can be dropped losslessly.
    fn duplicates(&self, other: &Self) -> bool {
        self.kind.to_lowercase() == other.kind.to_lowercase()
            && self.key.to_lowercase() == other.key.to_lowercase()
            && same_tags(&self.tags, &other.tags)
    }
}

/// Multiset equality for tag lists: the same tags in any order. O(n^2), but tag
/// counts per entry are small.
fn same_tags(left: &[Tag], right: &[Tag]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut matched = vec![false; right.len()];
    'left: for tag in left {
        for (j, other) in right.iter().enumerate() {
            if !matched[j] && tag == other {
                matched[j] = true;
                continue 'left;
            }
        }
        return false;
    }
    true
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Value;

    fn tag(name: &str, value: &str) -> Tag {
        Tag::new(name.to_string(), Value::Single(value.to_string()))
    }

    fn reference(key: &str, tags: Vec<Tag>) -> EntryType {
        EntryType::RefEntry(RefEntry::new("article".to_string(), key.to_string(), tags))
    }

    fn keys(entries: &Entries) -> Vec<String> {
        entries
            .iter()
            .filter_map(|e| match e {
                EntryType::RefEntry(r) => Some(r.key.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn collapses_exact_duplicates_keeping_first() {
        let mut entries = Entries::new(vec![
            reference("a", vec![tag("author", "First")]),
            reference("a", vec![tag("author", "First")]),
        ]);
        let warnings = entries.remove_duplicates();
        assert_eq!(keys(&entries), vec!["a"]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn ignores_tag_order_when_collapsing() {
        let mut entries = Entries::new(vec![
            reference("a", vec![tag("author", "X"), tag("year", "2020")]),
            reference("a", vec![tag("year", "2020"), tag("author", "X")]),
        ]);
        let warnings = entries.remove_duplicates();
        assert_eq!(keys(&entries), vec!["a"]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn matches_keys_case_insensitively() {
        let mut entries = Entries::new(vec![
            reference("Smith2020", vec![tag("author", "X")]),
            reference("smith2020", vec![tag("author", "X")]),
        ]);
        let warnings = entries.remove_duplicates();
        assert_eq!(keys(&entries), vec!["Smith2020"]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn keeps_conflicting_entries_and_warns() {
        let mut entries = Entries::new(vec![
            reference("a", vec![tag("author", "First")]),
            reference("a", vec![tag("author", "Second")]),
        ]);
        let warnings = entries.remove_duplicates();
        assert_eq!(keys(&entries), vec!["a", "a"]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("'a'"));
        assert!(warnings[0].contains('2'));
    }

    #[test]
    fn collapses_exact_dup_then_warns_on_remaining_conflict() {
        let mut entries = Entries::new(vec![
            reference("a", vec![tag("author", "First")]),
            reference("a", vec![tag("author", "Second")]),
            reference("a", vec![tag("author", "First")]),
        ]);
        let warnings = entries.remove_duplicates();
        // The third entry is an exact dup of the first and is dropped; the
        // second is a genuine conflict and is kept.
        assert_eq!(keys(&entries), vec!["a", "a"]);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn keeps_leading_whitespace_aligned() {
        let mut entries = Entries::with_leading(
            vec![
                reference("a", vec![tag("author", "X")]),
                reference("a", vec![tag("author", "X")]),
                reference("b", vec![tag("author", "Y")]),
            ],
            vec![String::new(), "\n\n".to_string(), "\n\n".to_string()],
        );
        entries.remove_duplicates();
        let pairs: Vec<(&str, String)> = entries
            .iter_with_leading()
            .filter_map(|(lead, e)| match e {
                EntryType::RefEntry(r) => Some((lead, r.key.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(
            pairs,
            vec![("", "a".to_string()), ("\n\n", "b".to_string())]
        );
    }

    #[test]
    fn leaves_non_reference_entries_untouched() {
        let mut entries = Entries::new(vec![
            EntryType::CommentEntry(CommentEntry::implicit("note".to_string())),
            EntryType::CommentEntry(CommentEntry::implicit("note".to_string())),
        ]);
        let warnings = entries.remove_duplicates();
        assert_eq!(entries.iter().count(), 2);
        assert!(warnings.is_empty());
    }
}
