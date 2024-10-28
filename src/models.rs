use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub struct Tag {
    pub name: String,
    pub content: String,
}

impl Tag {
    pub fn new(name: String, content: String) -> Self {
        Tag {
            name: name.to_lowercase(),
            content,
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {{{}}}", self.name, self.content)
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.name == other.name {
            return Ordering::Equal;
        }
        match self.name.as_str() {
            "title" => Ordering::Less,
            "author" => match other.name.as_str() {
                "title" => Ordering::Greater,
                _ => Ordering::Less,
            },
            _ => match other.name.as_str() {
                "title" | "author" => Ordering::Greater,
                _ => self.name.cmp(&other.name),
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Entry {
    pub kind: String,
    pub key: String,
    pub tags: Vec<Tag>,
}

impl Entry {
    pub fn new(kind: String, key: String, mut tags: Vec<Tag>) -> Self {
        tags.sort();
        Entry {
            kind: kind.to_lowercase(),
            key: key.to_lowercase(),
            tags,
        }
    }
}

impl fmt::Display for Entry {
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

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}
