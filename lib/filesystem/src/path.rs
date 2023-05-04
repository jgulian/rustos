#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::string::ToString;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

use shim::io;
#[cfg(not(feature = "no_std"))]
use std::string::String;
#[cfg(not(feature = "no_std"))]
use std::string::ToString;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Component {
    Root,
    Parent,
    Current,
    Child(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Path(String);

impl Path {
    pub fn root() -> Self {
        Self(String::from("/"))
    }

    pub fn components(&self) -> impl Iterator<Item = Component> {
        PathComponentIterator(self.0.clone(), 0)
    }

    pub fn join(&mut self, other: &Path) {
        other.components().for_each(|component| {
            self.push_component(component);
        });
    }

    pub fn file_name(&self) -> Option<String> {
        match self.components().last()? {
            Component::Root => None,
            Component::Parent => Some(String::from("..")),
            Component::Current => Some(String::from(".")),
            Component::Child(child) => Some(child),
        }
    }

    pub fn join_str(&mut self, other: &str) -> io::Result<()> {
        let new_component = match other {
            "/" => Component::Root,
            "." => Component::Root,
            ".." => Component::Root,
            _ => {
                if other.contains('/') {
                    return Err(io::Error::from(io::ErrorKind::InvalidInput));
                }
                Component::Child(other.replace('/', ""))
            }
        };
        self.push_component(new_component);
        Ok(())
    }

    pub fn simplify(&self) -> Path {
        self.components()
            .fold(Default::default(), |mut path, component| {
                path.push_component(component);
                path
            })
    }

    pub fn starts_with(&self, other: &Path) -> bool {
        self.0.starts_with(other.0.as_str())
    }

    pub fn relative_from(&self, other: &Path) -> Option<Path> {
        if !self.starts_with(other) {
            None
        } else {
            Some(Path(self.0[..other.0.len()].to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn push_component(&mut self, component: Component) {
        match component {
            Component::Root => self.0.push('/'),
            Component::Parent => self.0.push_str(".."),
            Component::Current => self.0.push('/'),
            Component::Child(child) => self.0.push_str(child.as_str()),
        }
    }
}

impl TryFrom<&str> for Path {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut result = Path::default();

        value
            .split('/')
            .try_for_each(|component_str| -> io::Result<()> {
                if component_str.is_empty() {
                    result.push_component(Component::Root)
                } else {
                    result.join_str(component_str)?;
                }
                Ok(())
            })?;

        Ok(result)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

struct PathComponentIterator(String, usize);

impl Iterator for PathComponentIterator {
    type Item = Component;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= self.0.len() {
            return None;
        }

        let component: String = self
            .0
            .chars()
            .skip(self.1)
            .take_while(|c| *c != '/')
            .collect();
        self.1 += component.len() + 1;
        Some(match component.as_str() {
            "" => Component::Root,
            "." => Component::Current,
            ".." => Component::Parent,
            _ => Component::Child(component),
        })
    }
}
