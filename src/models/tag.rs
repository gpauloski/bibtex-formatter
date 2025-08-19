use std::cmp::{Ord, Ordering, PartialOrd};

#[derive(Debug, Eq, PartialEq)]
pub struct Tag {
    pub name: String,
    pub value: Value,
}

impl Tag {
    pub const fn new(name: String, value: Value) -> Self {
        Self { name, value }
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        let this = self.name.to_lowercase();
        let them = other.name.to_lowercase();
        if this == them {
            return Ordering::Equal;
        }
        match this.as_str() {
            "title" => Ordering::Less,
            "author" => match them.as_str() {
                "title" => Ordering::Greater,
                _ => Ordering::Less,
            },
            _ => match them.as_str() {
                "title" | "author" => Ordering::Greater,
                _ => this.cmp(&them),
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

#[derive(Debug, Eq, PartialEq)]
pub struct Sequence(Vec<Part>);

impl Sequence {
    pub const fn new(parts: Vec<Part>) -> Self {
        Self(parts)
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(Part::is_empty)
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn parts(&self) -> &Vec<Part> {
        &self.0
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
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Quoted(s) | Self::Value(s) => s.is_empty(),
        }
    }
}
