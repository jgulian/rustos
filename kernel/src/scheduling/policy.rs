use core::time::Duration;

use crate::process::{ProcessId, State};
use crate::scheduling::scheduler::SwitchCondition;
use crate::scheduling::SwitchTrigger;

pub trait Policy: Send {
    fn insert(&mut self, process_id: ProcessId);
    fn remove(&mut self, process_id: ProcessId);

    fn inform(&mut self, process_id: ProcessId, information: &PolicyInformation);
    fn advise(&mut self) -> Option<PolicyAdvice>;
}

pub type Waiting = bool;

pub enum PolicyInformation {
    ScheduledOut(Duration, SwitchTrigger, Waiting),
    DoneWaiting,
    StartRunning,
}

pub struct PolicyAdvice {
    pub process_id: ProcessId,
    pub runtime: Option<Duration>,
    pub stop_condition: SwitchCondition,
}
