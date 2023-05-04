use alloc::collections::{BTreeSet, VecDeque};
use core::time::Duration;

use kernel_api::println;

use crate::console::kprintln;
use crate::process::ProcessId;
use crate::scheduling::policy::{Policy, PolicyAdvice, PolicyInformation};
use crate::scheduling::scheduler::SwitchCondition;
use crate::scheduling::SwitchTrigger;

pub struct RoundRobinPolicy {
    quantum: Duration,
    queue: VecDeque<ProcessId>,
    waiting: BTreeSet<ProcessId>,
}

impl RoundRobinPolicy {
    pub fn new(quantum: Duration) -> Self {
        Self {
            quantum,
            queue: VecDeque::new(),
            waiting: BTreeSet::new(),
        }
    }

    fn index_of_process_id(&self, process_id: ProcessId) -> Option<usize> {
        self.queue.iter().position(|id| id == &process_id)
    }
}

impl Policy for RoundRobinPolicy {
    fn insert(&mut self, process_id: ProcessId) {
        self.queue.push_front(process_id);
    }

    fn remove(&mut self, process_id: ProcessId) {
        if let Some(i) = self.index_of_process_id(process_id) {
            self.queue.remove(i);
        }
    }

    fn inform(&mut self, process_id: ProcessId, information: &PolicyInformation) {
        match information {
            PolicyInformation::ScheduledOut(_, _, waiting) => {
                if *waiting {
                    self.waiting.insert(process_id);
                }
                self.queue.push_back(process_id);
            }
            PolicyInformation::DoneWaiting => {
                self.waiting.remove(&process_id);
            }
            PolicyInformation::StartRunning => {
                self.queue.remove(
                    self.queue
                        .iter()
                        .position(|pid| *pid == process_id)
                        .unwrap(),
                );
            }
        }

        if let Some(i) = self.index_of_process_id(process_id) {
            self.queue.remove(i);
            self.queue.push_back(process_id);
        }
    }

    fn advise(&mut self) -> Option<PolicyAdvice> {
        Some(PolicyAdvice {
            process_id: *self
                .queue
                .iter()
                .find(|process_id| !self.waiting.contains(process_id))?,
            runtime: Some(self.quantum),
            stop_condition: SwitchCondition::new().or(SwitchTrigger::Yield),
        })
    }
}
