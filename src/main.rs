use crate::app::App;
use color_eyre::Result;
use std::time::Duration;

mod app;
mod args;
mod entry;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::new();

    let terminal = ratatui::init();
    app.run(terminal, Duration::from_millis(500))?;

    ratatui::restore();
    Ok(())
}
