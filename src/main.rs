pub use app::App;

pub mod app;

fn main() -> anyhow::Result<()> {
    let terminal = ratatui::init();
    App::new()?.run(terminal)?;
    ratatui::restore();
    Ok(())
}
