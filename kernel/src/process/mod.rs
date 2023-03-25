pub use crate::param::TICK;

pub use self::process::{ProcessId, Process};
pub use self::state::State;
pub use self::resource::ResourceId;


mod process;
mod state;
mod resource;
mod pipe;

