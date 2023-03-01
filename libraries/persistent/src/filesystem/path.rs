use alloc::vec::Vec;

#[derive(Copy)]
pub enum PathComponent {
    Root,
    Current,
    Parent,
    Child(String),
}

pub struct Path(Vec<PathComponent>);

impl Default for Path {
    fn default() -> Self {
        Path(vec![PathComponent::Root])
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        self.0.iter().fold(String::default(), |mut result, component| {
            match component {
                PathComponent::Root => {
                    result.push_str("//");
                }
                PathComponent::Current => {
                    result.push_str("./")
                }
                PathComponent::Parent => {
                    result.push_str("../")
                }
                PathComponent::Child(child) => {
                    result.push_str(child.as_str());
                }
            }

            result.pop();
            result
        })
    }
}

impl Path {
    fn push(&mut self, child: &str) {
        self.0.push(PathComponent::Child(child.to_string()));
    }

    fn push_component(&mut self, component: PathComponent) {
        self.0.push(component);
    }
}

macro_rules! path {

}