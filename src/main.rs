use process::Process;
use std::error::Error;
use sysinfo::System;
use ui::run_ui;

mod process;
mod tree;
mod ui;

type R<A> = Result<A, Box<dyn Error>>;

fn main() -> R<()> {
    let system = System::new();
    let tree = Process::new_from_sysinfo(
        system
            .processes()
            .values()
            .filter(|process| process.thread_kind().is_none()),
    );
    run_ui(tree, system)?;
    Ok(())
}
