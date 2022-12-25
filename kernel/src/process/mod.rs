pub use crate::param::TICK;

pub use self::process::{Id, Process};
pub use self::scheduler::GlobalScheduler;
pub use self::stack::Stack;
pub use self::state::State;
pub use self::resource::ResourceId;

mod process;
mod scheduler;
mod stack;
mod state;
mod resource;
mod pipe;

