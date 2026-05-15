use clap::Parser;
use pad::app::App;
use pad::cli::{Args, Mode};
use pad::config::paths;
use pad::panic_hook::{file_sink, install_panic_hook};
use pad::tui::Tui;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    install_panic_hook(file_sink(paths::state_root()));
    match args.mode() {
        Mode::Recover => {
            pad::recover::run(&paths::state_root())?;
            Ok(())
        }
        mode => {
            let mut tui = Tui::enter()?;
            let mut app = App::from_mode(mode)?;
            app.run(&mut tui).await
        }
    }
}
