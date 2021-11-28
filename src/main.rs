mod mrs;
mod rb;

use anyhow;
use clap::{App, Arg};

pub type Res<T> = anyhow::Result<T>;

#[derive(PartialOrd, PartialEq)]
pub enum Verbosity {
    Normal = 0,
    Info,
    Debug,
}

fn main() -> Res<()> {
    let mut app = App::new("rustgit")
        .version("1.0")
        .author("Dan Farino")
        .about("Dan's Rust-based Git utility")
        .subcommand(App::new("rb").about("Shows recently-used branches by looking at the reflog"))
        .subcommand(
            App::new("mrs").about("Multi-repo status").arg(
                Arg::new("verbose")
                    .long("verbose")
                    .short('v')
                    .multiple_occurrences(true),
            ),
        );

    let matches = app.clone().get_matches();

    if let Some(_) = matches.subcommand_matches("rb") {
        return rb::command_rb();
    }

    if let Some(sm) = matches.subcommand_matches("mrs") {
        let verbosity = match sm.occurrences_of("verbose") {
            0 => Verbosity::Normal,
            1 => Verbosity::Info,
            _ => Verbosity::Debug,
        };
        return mrs::command_multi_repo_status(verbosity);
    }

    app.print_help().unwrap();
    Ok(())
}
