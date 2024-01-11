use process::Process;
use std::env::args;
use std::error::Error;
use std::io::stdout;
use std::io::Write;
use sysinfo::System;

mod process;
mod tree;

type R<A> = Result<A, Box<dyn Error>>;

fn main() -> R<()> {
    let pattern = args().nth(1).unwrap_or("".to_string());
    let system = System::new_all();
    stdout().write_all(
        Process::new_from_sysinfo(
            system
                .processes()
                .values()
                .filter(|process| process.thread_kind().is_none()),
        )
        .format(|p| p.name.contains(pattern.as_str()))
        .as_bytes(),
    )?;
    Ok(())
}
