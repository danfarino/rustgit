mod mrs;
mod rb;

use anyhow;
use clap::App;

pub type Res<T> = anyhow::Result<T>;

fn main() -> Res<()> {
    let mut app = App::new("rustgit")
        .version("1.0")
        .author("Dan Farino")
        .about("Dan's Rust-based Git utility")
        .subcommand(App::new("rb").about("Shows recently-used branches by looking at the reflog"))
        .subcommand(App::new("mrs").about("Multi-repo status"));

    let matches = app.clone().get_matches();

    if let Some(_) = matches.subcommand_matches("rb") {
        return rb::command_rb();
    }

    if let Some(_) = matches.subcommand_matches("mrs") {
        return mrs::command_multi_repo_status();
    }

    app.print_help().unwrap();
    Ok(())
}
