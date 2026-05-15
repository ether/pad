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
        Mode::Restore => {
            pad::recover::run_restore(&paths::state_root())?;
            Ok(())
        }
        Mode::Setup => run_setup().await,
        Mode::JoinUrl(url) => {
            let parsed = pad::share::url_parse::parse_pad_url(&url)?;
            let mut tui = Tui::enter()?;
            let mut app = App::from_join_url(parsed).await?;
            app.run(&mut tui).await
        }
        mode => {
            let mut tui = Tui::enter()?;
            let mut app = App::from_mode(mode)?;
            app.run(&mut tui).await
        }
    }
}

async fn run_setup() -> anyhow::Result<()> {
    let instances = pad::share::scanner::fetch_or_fallback().await;
    println!("Pick an Etherpad instance to use as your default remote:");
    for (i, inst) in instances.iter().enumerate() {
        println!("  [{}] {} ({})", i + 1, inst.label, inst.url);
    }
    print!("> ");
    use std::io::Write;
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let idx: usize = input.trim().parse().unwrap_or(0);
    if idx == 0 || idx > instances.len() {
        eprintln!("invalid selection");
        std::process::exit(1);
    }
    let chosen = &instances[idx - 1];
    let mut cfg = pad::config::Config::load().unwrap_or_default();
    cfg.remote = Some(chosen.url.clone());
    if !cfg.consented_remotes.contains(&chosen.url) {
        cfg.consented_remotes.push(chosen.url.clone());
    }
    cfg.save()?;
    println!("Saved remote = {}", chosen.url);
    Ok(())
}
