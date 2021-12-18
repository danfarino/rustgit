mod mrs;
mod rb;

use clap::{App, Arg};

pub type Res<T> = anyhow::Result<T>;

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum Verbosity {
    Normal = 0,
    Info,
    Debug,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
        )
        .subcommand(App::new("version").about("Show version number"));

    let matches = app.clone().get_matches();

    if matches.subcommand_matches("version").is_some() {
        println!("{}", VERSION);
        return Ok(());
    }

    if matches.subcommand_matches("rb").is_some() {
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
