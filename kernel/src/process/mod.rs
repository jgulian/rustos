pub use crate::param::TICK;

pub use self::process::{Process, ProcessId};
pub use self::resource::ResourceId;
pub use self::state::State;

mod pipe;
mod process;
mod resource;
mod state;
