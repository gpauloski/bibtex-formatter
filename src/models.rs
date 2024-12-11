use crate::Result;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Eq, PartialEq)]
pub enum Content {
    Braced(String),
    Quoted(String),
    Value(String),
}

impl Content {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Braced(s) | Self::Quoted(s) | Self::Value(s) => s.trim().is_empty(),
        }
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Braced(s) => write!(f, "{{{}}}", s),
            Self::Quoted(s) => write!(f, "\"{}\"", s),
            Self::Value(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ContentParts(Vector<Content>);

impl fmt::Display for ContentParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = match self.0.len() {
            0 => "{}".to_string(),
            1 => match self.0.first().unwrap() {
                Content::Braced(s) | Content::Quoted(s) => format!("{{{}}}", s),
                Content::Value(s) => s.clone(),
            }
            _ => {
                self.0
                    .iter()
                    .map(|part| part.to_string())
                    .collect::<Vec<String>>()
                    .join(" # ")
            },
        };
        write!(f, "{}", content);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Tag {
    pub name: String,
    pub content: ContentParts,
}

impl Tag {
    pub fn new(name: String, content: Vec<Content>) -> Self {
        Tag { name, ContentParts(content) }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name.to_lowercase(), content)
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

#[derive(Debug)]
pub enum Entry {
    RefEntry(RefEntry),
    StringEntry(StringEntry),
}

#[derive(Debug)]
pub struct Entries {
    pub references: Vec<RefEntry>,
    pub strings: Vec<StringEntry>,
}

impl Entries {
    pub fn sort(&mut self) {
        self.references.sort();
        self.strings.sort();
    }

    pub fn write(&self, filepath: &str) -> Result<()> {
        let mut file = File::create(filepath)?;
        write!(file, "{}", self)?;
        Ok(())
    }
}

impl fmt::Display for Entries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, string) in self.strings.iter().enumerate() {
            if i == 0 {
                write!(f, "{}", string)?;
            } else {
                write!(f, "\n{}", string)?;
            }
        }

        for (i, reference) in self.references.iter().enumerate() {
            if i == 0 && self.strings.is_empty() {
                write!(f, "{}", reference)?;
            } else {
                write!(f, "\n\n{}", reference)?;
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

impl fmt::Display for RefEntry {
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
pub struct StringEntry {
    pub name: String,
    pub content: ContentParts,
}

impl StringEntry {
    pub fn new(name: String, content: Vec<Content>) -> Self {
        Self { name, ContentParts(content) }
    }
}

impl fmt::Display for StringEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@STRING{{{} = \"{}\"}}", self.name, self.content.trim())
    }
}

impl PartialOrd for StringEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StringEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
