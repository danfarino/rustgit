mod age;
mod mrs;

use anyhow;
use chrono::TimeZone;
use clap::App;
use git2::{BranchType, ErrorCode, Repository};
use std::collections::HashSet;
use std::ops::Sub;

type Res<T> = anyhow::Result<T>;

fn main() -> Res<()> {
    let mut app = App::new("rustgit")
        .version("1.0")
        .author("Dan Farino")
        .about("Dan's Rust-based Git utility")
        .subcommand(App::new("rb").about("Shows recently-used branches by looking at the reflog"))
        .subcommand(App::new("mrs").about("Multi-repo status"));

    let matches = app.clone().get_matches();

    if let Some(_) = matches.subcommand_matches("rb") {
        return command_rb();
    }

    if let Some(_) = matches.subcommand_matches("mrs") {
        return mrs::command_multi_repo_status();
    }

    app.print_help().unwrap();
    Ok(())
}

struct BranchInfo {
    name: String,
    seen: String,
    ago: String,
}

fn command_rb() -> Res<()> {
    let repo = Repository::discover(std::env::current_dir()?).map_err(|e| match e.code() {
        ErrorCode::NotFound => anyhow::anyhow!("Please run this from inside of a Git repo"),
        _ => anyhow::Error::from(e),
    })?;

    let mut seen_branches = HashSet::new();
    let re = regex::Regex::new(r"^(?:checkout:.+?|Branch: renamed .+ to refs/heads/)(\S+)$")?;

    let mut branch_infos = Vec::new();

    for entry in repo.reflog("HEAD")?.iter() {
        let msg = entry.message().unwrap();

        if let Some(cap) = re.captures(msg).as_ref() {
            let branch_name = &cap[1];

            if seen_branches.insert(branch_name.to_owned()) {
                if repo.find_branch(branch_name, BranchType::Local).is_err() {
                    // branch not found
                    continue;
                }

                let secs = entry.committer().when().seconds();
                let dt = chrono::Utc.timestamp(secs, 0);
                let local = chrono::DateTime::from(dt) as chrono::DateTime<chrono::Local>;
                let dur = dt.sub(chrono::Utc::now());
                let ago = age::format_age(&dur);

                branch_infos.push(BranchInfo {
                    name: branch_name.to_owned(),
                    seen: local.to_rfc2822(),
                    ago,
                });
            }
        }
    }

    if branch_infos.is_empty() {
        println!("No relevant entries found in reflog.");
        return Ok(());
    }

    let col1 = branch_infos.iter().map(|o| o.name.len()).max().unwrap();
    let col2 = branch_infos.iter().map(|o| o.ago.len()).max().unwrap();

    let (ansi1, ansi2) = if atty::is(atty::Stream::Stdout) {
        ("\x1b[32;1m", "\x1b[m")
    } else {
        ("", "")
    };

    for entry in branch_infos.iter() {
        println!(
            "{}{:col1$}{}   {:col2$}   {}",
            ansi1,
            entry.name,
            ansi2,
            entry.ago,
            entry.seen,
            col1 = col1,
            col2 = col2
        );
    }

    Ok(())
}
