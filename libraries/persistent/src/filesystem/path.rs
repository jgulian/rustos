use

pub enum PathComponent {
    Root,
    Current,
    Parent,
    Child(String),
}

pub struct Path(Vec<PathComponent>);

impl Path {

}

