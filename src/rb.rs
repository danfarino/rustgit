use crate::Res;
use chrono::TimeZone;
use git2::{BranchType, ErrorCode, Repository};
use std::collections::HashSet;
use std::ops::Sub;

const MINUTE: f64 = 60.;
const HOUR: f64 = MINUTE * 60.;
const DAY: f64 = HOUR * 24.;
const WEEK: f64 = DAY * 7.;
const MONTH: f64 = DAY * (365. / 12.);
const YEAR: f64 = MONTH * 12.;

struct BranchInfo {
    name: String,
    seen: String,
    ago: String,
}

pub fn command_rb() -> Res<()> {
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
                let dt = chrono::Utc.timestamp_opt(secs, 0).unwrap();
                let local = chrono::DateTime::from(dt) as chrono::DateTime<chrono::Local>;
                let dur = dt.sub(chrono::Utc::now());
                let ago = format_age(&dur);

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

fn format(n: f64, unit: f64, word: &str) -> String {
    let n = (n / unit).floor() as i64;
    let mut word = word.to_owned();
    if n != 1 {
        word += "s"
    }
    format!("{} {}", n, word)
}

pub fn format_age(dur: &chrono::Duration) -> String {
    match dur.num_seconds().abs() as f64 {
        n if n > YEAR => format(n, YEAR, "year"),
        n if n > MONTH => format(n, MONTH, "month"),
        n if n > WEEK => format(n, WEEK, "week"),
        n if n > DAY => format(n, DAY, "day"),
        n if n > HOUR => format(n, HOUR, "hour"),
        n if n > MINUTE => format(n, MINUTE, "minute"),
        n => format(n, 1., "second"),
    }
}
