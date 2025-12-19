use crate::{app::App, args::Args};
use clap::Parser;
use color_eyre::Result;
use std::time::Duration;

mod app;
mod args;
mod entry;

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let mut app = App::new(args);

    let terminal = ratatui::init();
    app.run(terminal, Duration::from_millis(500))?;

    ratatui::restore();
    Ok(())
}
