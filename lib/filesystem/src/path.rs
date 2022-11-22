use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use core::borrow::Borrow;

use shim::{io, ioerr};

use crate::path::Component::Parent;

pub struct Path(Vec<Component>);

#[derive(Clone, PartialEq)]
pub enum Component {
    Root,
    Current,
    Parent,
    Child(String),
}

impl Path {
    pub fn new(path: &str) -> io::Result<Self> {
        Path::try_from(path)
    }

    pub fn root() -> Self {
        Path(vec![Component::Root])
    }

    pub fn append(&mut self, path: &Path) {
        self.0.extend_from_slice(path.0.as_slice());
    }

    pub fn components(&self) -> Vec<Component> {
        self.0.clone()
    }

    pub fn simplify(&self) -> io::Result<Path> {
        let mut simplified = Vec::new();

        for component in &self.0 {
            match component {
                Component::Root => {
                    simplified.clear();
                    simplified.push(Component::Root);
                }
                Component::Current => {}
                Parent => {
                    match simplified.last() {
                        Some(last) if *last != Component::Root => {
                            simplified.pop();
                        }
                        _ => {
                            return ioerr!(InvalidFilename);
                        }
                    }
                }
                Component::Child(child) => {
                    simplified.push(Component::Child(child.clone()))
                }
            }
        }

        Ok(Path(simplified))
    }
}

impl TryFrom<&str> for Path {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut result = Vec::new();
        let mut children = value.split("/").peekable();
        while let Some(child) = children.next() {
            match child {
                "" => {
                    if children.peek().is_some() {
                        result.push(Component::Root)
                    }
                }
                "." => {
                    result.push(Component::Current);
                }
                ".." => {
                    result.push(Component::Parent)
                }
                _ => {
                    if !is_valid_entry(child) {
                        return ioerr!(InvalidFilename);
                    } else {
                        result.push(Component::Child(child.to_string()))
                    }
                }
            }
        }

        Ok(Path(result))
    }
}

impl TryFrom<String> for Path {
    type Error = io::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Path::try_from(value.as_str())
    }
}

impl From<&[Component]> for Path {
    fn from(value: &[Component]) -> Self {
        Path(Vec::from(value))
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        let mut result = String::new();
        let mut components = self.0.iter().peekable();
        while let Some(component) = components.next() {
            match component {
                Component::Root => {
                    result.push('/');
                }
                Component::Current => {
                    result.push_str("./");
                }
                Parent => {
                    result.push_str("./");
                }
                Component::Child(child) => {
                    result.push_str(child.as_str());
                    if components.peek().is_some() {
                        result.push('/');
                    }
                }
            }
        }

        result
    }
}

fn is_valid_entry(name: &str) -> bool {
    name.chars().all(|c| {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | ' ' | '.' => true,
            _ => false,
        }
    })
}