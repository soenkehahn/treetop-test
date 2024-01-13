use process::Process;
use std::error::Error;
use std::thread;
use std::time::Duration;
use sysinfo::System;
use ui::run_ui;

mod process;
mod tree;
mod ui;

type R<A> = Result<A, Box<dyn Error>>;

fn main() -> R<()> {
    let mut system = System::new_all();
    eprintln!("measuring cpu usage...");
    thread::sleep(Duration::from_secs(1));
    system.refresh_all();
    let tree = Process::new_from_sysinfo(
        system
            .processes()
            .values()
            .filter(|process| process.thread_kind().is_none()),
    );
    run_ui(tree)?;
    Ok(())
}
