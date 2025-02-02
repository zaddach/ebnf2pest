use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct Args {
    pub file: PathBuf,
}