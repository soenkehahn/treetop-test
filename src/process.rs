use crate::tree::Node;
use crate::tree::Tree;
use std::collections::HashMap;
use std::fmt;
use sysinfo::Pid;
use sysinfo::ThreadKind;

#[derive(Clone, Debug)]
pub(crate) struct Process {
    pid: Pid,
    name: String,
    parent: Option<Pid>,
    kind: Option<ThreadKind>,
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} : {} - {:?}", self.pid, self.name, self.kind)
    }
}

impl Node<Pid> for Process {
    fn root() -> Pid {
        Pid::from_u32(1)
    }

    fn id(&self) -> Pid {
        self.pid
    }

    fn parent(&self) -> Option<Pid> {
        self.parent
    }
}

impl Process {
    fn from_sysinfo_process(process: &sysinfo::Process) -> Self {
        Process {
            pid: process.pid(),
            name: process.name().to_string(),
            parent: process.parent(),
            kind: process.thread_kind(),
        }
    }

    pub(crate) fn new_from_sysinfo(processes: &HashMap<Pid, sysinfo::Process>) -> Tree<Pid, Self> {
        Tree::new(processes.values().map(Process::from_sysinfo_process))
    }
}
