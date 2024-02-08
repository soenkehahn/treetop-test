use porc_app::run_ui;
use std::error::Error;
use sysinfo::System;

mod porc_app;
mod process;
mod tree;
mod tui_app;

type R<A> = Result<A, Box<dyn Error>>;

fn main() -> R<()> {
    run_ui(System::new())
}
