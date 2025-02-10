use crate::app::App;

mod app;
mod todo;
mod ui_state;

fn main() -> anyhow::Result<()> {
    let terminal = ratatui::init();
    App::new()?.run(terminal)?;
    ratatui::restore();
    Ok(())
}
