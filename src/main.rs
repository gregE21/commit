#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::str_to_string
)]
#![allow(clippy::module_name_repetitions)]

mod commit;
mod commit_message;
mod config;

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use commit_message::make_message_commit;

const DEFAULT_CONFIG_FILE: &str = include_str!("../commit-default.json");

#[derive(Parser)]
#[clap(about, author, version)]
struct Opt {
    #[clap(
        short,
        long,
        help = "Custom configuration file path",
        parse(from_os_str)
    )]
    config: Option<PathBuf>,
    #[clap(long, help = "Init custom configuration file")]
    init: bool,
    #[clap(short, long, help = "Use as hook")]
    hook: bool,
}

fn main() -> Result<()> {
    // Dumb hack to set the current/working directory (pwd) because appimage or cargo-appimage sucks
    // https://github.com/AppImage/AppImageKit/issues/172
    if let Ok(current_dir) = std::env::var("OWD") {
        std::env::set_current_dir(current_dir)?;
    }

    if let Some(0) = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .output()
        .expect("failed to execute process")
        .status
        .code()
    {
        return Err(anyhow!(
            "{}",
            "You have not added anything please do `git add`".red()
        ));
    }

    let opt = Opt::parse();
    if opt.init {
        let mut file = std::fs::File::create("commit.json")?;
        file.write_all(DEFAULT_CONFIG_FILE.as_bytes())?;
        return Ok(());
    }

    let pattern = config::get_pattern(opt.config)?;
    let commit_message = make_message_commit(pattern)?;

    if opt.hook {
        commit::commit_as_hook(&commit_message)?;
        return Ok(());
    }

    commit::commit(&commit_message)?;
    Ok(())
}
