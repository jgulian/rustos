use alloc::collections::{BinaryHeap, BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::cmp::{max, Ordering};
use core::ops::{Add, Div};
use core::time::Duration;

use crate::process::ProcessId;
use crate::scheduling::policy::{Policy, PolicyAdvice, PolicyInformation};
use crate::scheduling::scheduler::SwitchCondition;
use crate::scheduling::SwitchTrigger;
use crate::scheduling::SwitchTrigger::{Timer, Yield};

pub struct FairPolicy {
    processes: BinaryHeap<ProcessInformation>,
    waiting: BTreeMap<ProcessId, Duration>,
    running: BTreeMap<ProcessId, Duration>,
    removed: BTreeSet<ProcessId>,
    scheduler_latency: Duration,
    minimum_granularity: Duration,
}

impl FairPolicy {
    pub(super) fn new(scheduler_latency: Duration, minimum_granularity: Duration) -> Self {
        Self {
            processes: BinaryHeap::new(),
            waiting: BTreeMap::new(),
            running: BTreeMap::new(),
            removed: BTreeSet::new(),
            scheduler_latency,
            minimum_granularity,
        }
    }

    fn min_duration(&self) -> Duration {
        self.processes
            .iter()
            .min_by(|a, b| a.runtime.cmp(&b.runtime))
            .map(|process_information| process_information.runtime)
            .unwrap_or(Duration::ZERO)
    }

    fn recommended_runtime(&self) -> Duration {
        if self.processes.is_empty() {
            self.minimum_granularity
        } else {
            max(
                self.minimum_granularity,
                self.scheduler_latency.div(self.processes.len() as u32),
            )
        }
    }
}

impl Policy for FairPolicy {
    fn insert(&mut self, process_id: ProcessId) {
        self.processes.push(ProcessInformation {
            process_id,
            runtime: self.min_duration(),
        });
    }

    fn remove(&mut self, process_id: ProcessId) {
        self.removed.insert(process_id);
    }

    fn inform(&mut self, process_id: ProcessId, information: &PolicyInformation) {
        match information {
            PolicyInformation::ScheduledOut(duration, _, waiting) => {
                let runtime = self.running.remove(&process_id).unwrap();
                let total_runtime = runtime + *duration;
                if *waiting {
                    self.waiting.insert(process_id, total_runtime);
                } else {
                    self.processes.push(ProcessInformation {
                        process_id,
                        runtime: total_runtime,
                    })
                }
            }
            PolicyInformation::DoneWaiting => {
                let runtime = self.waiting.remove(&process_id).unwrap();
                self.processes.push(ProcessInformation {
                    process_id,
                    runtime,
                })
            }
            PolicyInformation::StartRunning => {
                let removed = if self.processes.peek().unwrap().process_id == process_id {
                    self.processes.pop().unwrap()
                } else {
                    let mut processes: Vec<_> = self.processes.iter().copied().collect();
                    let process = processes.remove(
                        processes
                            .iter()
                            .position(|process_information| {
                                process_information.process_id == process_id
                            })
                            .unwrap(),
                    );
                    self.processes = BinaryHeap::from(processes);
                    process
                };
                self.running.insert(removed.process_id, removed.runtime);
            }
        }
    }

    fn advise(&mut self) -> Option<PolicyAdvice> {
        while let Some(process_information) = self.processes.peek() {
            if self.removed.contains(&process_information.process_id) {
                self.removed.remove(&process_information.process_id);
                self.processes.pop();
            } else {
                break;
            }
        }

        self.processes
            .peek()
            .map(|process_information| PolicyAdvice {
                process_id: process_information.process_id.clone(),
                runtime: Some(self.recommended_runtime()),
                stop_condition: SwitchCondition::new().or(Timer).or(Yield),
            })
    }
}

#[derive(Copy, Clone)]
struct ProcessInformation {
    process_id: ProcessId,
    runtime: Duration,
}

impl Eq for ProcessInformation {}

impl PartialEq<Self> for ProcessInformation {
    fn eq(&self, other: &Self) -> bool {
        self.runtime.eq(&other.runtime)
    }
}

impl PartialOrd<Self> for ProcessInformation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProcessInformation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.runtime.cmp(&other.runtime)
    }
}
