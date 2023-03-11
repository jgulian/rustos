#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(not(feature = "no_std"))]
use std::string::String;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Component {
    Root,
    Parent,
    Current,
    Child(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Path(String);

impl Path {
    fn new(components: Vec<Component>) -> Path {
        let mut path = Path(String::new());
        for component in components {
            path.push_component(component)
        }
        path
    }

    pub fn components(&self) -> impl Iterator<Item=Component> {
        PathComponentIterator(self.0.clone(), 0)
    }

    pub fn join(&mut self, other: &Path) {
        other.components().for_each(|component| {
            self.push_component(component);
        });
    }

    pub fn join_str(&mut self, other: &str) {
        let new_component = match other {
            "/" => Component::Root,
            "." => Component::Root,
            ".." => Component::Root,
            _ => Component::Child(other.replace("/", ""))
        };
        self.push_component(new_component);
    }

    pub fn simplify(&self) -> Path {
        self.components().fold(Default::default(), |mut path, component| {
            path.push_component(component);
            path
        })
    }

    pub fn starts_with(&self, other: &Path) -> bool {
        let mut other_components = other.components();
        let equal = self.components().zip(other.components())
            .all(|(a, b)| a == b);
        equal && other_components.next().is_none()
    }

    pub fn relative_from(&self, other: &Path) -> Option<Path> {
        let mut self_components = self.components();
        let prefixed = other.components().all(|component| {
            if let Some(self_component) = self_components.next() {
                component == self_component
            } else {
                false
            }
        });

        if prefixed {
            Some(Self::new(self_components.collect()))
        } else {
            None
        }
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

impl Default for Path {
    fn default() -> Self {
        Self(String::from("/"))
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

        let component: String = self.0.chars().skip(self.1).take_while(|c| *c != '/').collect();
        self.1 += component.len() + 1;
        Some(
            match component.as_str() {
                "" => Component::Root,
                "." => Component::Current,
                ".." => Component::Parent,
                _ => Component::Child(component),
            }
        )
    }
}