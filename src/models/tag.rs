use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub struct Tag {
    pub name: String,
    pub value: Value,
}

impl Tag {
    pub fn new(name: String, value: Value) -> Self {
        Tag {
            name: name.to_lowercase(),
            value,
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.value)
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
pub enum Value {
    Single(String),
    Integer(u64),
    Sequence(Sequence),
}

impl Value {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(s) => s.trim().is_empty(),
            Self::Integer(_) => false,
            Self::Sequence(s) => s.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(s) => write!(f, "{{{}}}", s),
            Self::Integer(s) => write!(f, "{}", s),
            Self::Sequence(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Sequence(Vec<Part>);

impl Sequence {
    pub fn new(parts: Vec<Part>) -> Self {
        Self(parts)
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|part| part.is_empty())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = self
            .0
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<String>>()
            .join(" # ");
        write!(f, "{}", content)
    }
}

impl Iterator for Sequence {
    type Item = Part;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Part {
    Quoted(String),
    Value(String),
}

impl Part {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Quoted(s) | Self::Value(s) => s.is_empty(),
        }
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quoted(s) => write!(f, "\"{}\"", s),
            Self::Value(v) => write!(f, "{}", v.to_lowercase()),
        }
    }
}
