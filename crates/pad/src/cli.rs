use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "pad", version, about = "Nano-class terminal text editor.")]
pub struct Args {
    /// Path to open. If omitted, opens an untitled buffer.
    pub path: Option<PathBuf>,

    /// List buffers with unsaved crash state and let you resume one.
    #[arg(long, conflicts_with = "path")]
    pub recover: bool,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Untitled,
    OpenFile(PathBuf),
    Recover,
}

impl Args {
    pub fn mode(&self) -> Mode {
        if self.recover {
            Mode::Recover
        } else if let Some(p) = &self.path {
            Mode::OpenFile(p.clone())
        } else {
            Mode::Untitled
        }
    }
}
