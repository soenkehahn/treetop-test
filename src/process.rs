use crate::tree::Forest;
use crate::tree::Node;
use std::fmt;
use sysinfo::Pid;

#[derive(Debug)]
pub(crate) struct Process {
    pid: Pid,
    pub(crate) name: String,
    parent: Option<Pid>,
    cpu: f32,
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Node for Process {
    type Id = Pid;

    fn id(&self) -> Pid {
        self.pid
    }

    fn format_table(&self) -> String {
        format!("{:>8} {:>4.0}%", self.pid.as_u32(), self.cpu)
    }

    fn parent(&self) -> Option<Pid> {
        self.parent
    }

    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cpu
            .partial_cmp(&self.cpu)
            .unwrap_or(self.pid.cmp(&other.pid))
    }

    fn accumulate_from(&mut self, other: &Self) {
        self.cpu += other.cpu;
    }
}

impl Process {
    fn from_sysinfo_process(process: &sysinfo::Process) -> Self {
        Process {
            pid: process.pid(),
            name: process.name().to_string(),
            parent: process.parent(),
            cpu: process.cpu_usage(),
        }
    }

    pub(crate) fn new_from_sysinfo<'a>(
        processes: impl Iterator<Item = &'a sysinfo::Process>,
    ) -> Forest<Self> {
        Forest::new_forest(processes.map(Process::from_sysinfo_process))
    }
}
