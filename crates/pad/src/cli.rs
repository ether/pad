use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "pad", version, about = "Nano-class terminal text editor.")]
pub struct Args {
    /// Path to open, or an http(s) URL to join a remote pad.
    /// If omitted, opens an untitled buffer.
    pub path: Option<PathBuf>,

    /// List buffers with unsaved crash state and let you resume one.
    #[arg(long, conflicts_with_all = ["path", "restore", "setup"])]
    pub recover: bool,

    /// List pre-share / pre-merge snapshots.
    #[arg(long, conflicts_with_all = ["path", "recover", "setup"])]
    pub restore: bool,

    /// Interactive first-run remote configuration.
    #[arg(long, conflicts_with_all = ["path", "recover", "restore"])]
    pub setup: bool,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Untitled,
    OpenFile(PathBuf),
    JoinUrl(String),
    Recover,
    Restore,
    Setup,
}

impl Args {
    pub fn mode(&self) -> Mode {
        if self.recover {
            Mode::Recover
        } else if self.restore {
            Mode::Restore
        } else if self.setup {
            Mode::Setup
        } else if let Some(p) = &self.path {
            let s = p.to_string_lossy();
            if s.starts_with("http://") || s.starts_with("https://") {
                Mode::JoinUrl(s.into_owned())
            } else {
                Mode::OpenFile(p.clone())
            }
        } else {
            Mode::Untitled
        }
    }
}
