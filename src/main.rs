use process::Process;
use std::error::Error;
use std::io::stdout;
use std::io::Write;
use sysinfo::System;

mod process;
mod tree;

type R<A> = Result<A, Box<dyn Error>>;

fn main() -> R<()> {
    let _pattern = "alacritty";
    let system = System::new_all();
    stdout().write_all(
        Process::new_from_sysinfo(system.processes())
            .format()
            .as_bytes(),
    )?;
    Ok(())
}
