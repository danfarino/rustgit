use crate::{Res, Verbosity};
use git2::{Branch, BranchType, ErrorCode, Repository, StatusOptions};
use glob::glob;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn command_multi_repo_status(verbosity: Verbosity) -> Res<()> {
    let enable_ansi_colors = atty::is(atty::Stream::Stdout);

    let homedir = dirs::home_dir().ok_or(anyhow::anyhow!("cannot locate user home dir"))?;
    let homedir_display = homedir
        .display()
        .to_string()
        .trim_end_matches("/")
        .to_string();

    let config_path = homedir.join(".rustgitrc");
    let config_contents = fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("cannot open {}: {}", &config_path.display(), e))?;

    let mut repos_paths = Vec::new();

    for config_line in config_contents.lines() {
        let repo_glob = shellexpand::tilde(config_line).to_string();

        let mut matched = false;

        for res in glob(&repo_glob)? {
            let path = res?;
            repos_paths.push(path);
            matched = true;
        }

        if !matched {
            repos_paths.push(PathBuf::from(&repo_glob));
        }
    }

    repos_paths.sort();
    repos_paths.dedup();

    for repo_path in &repos_paths {
        if verbosity >= Verbosity::Info {
            println!("Checking repo: {}", &repo_path.display());
        }

        let res = process_repo(repo_path);
        if let Err(e) = res {
            let (ansi1, ansi2) = if enable_ansi_colors {
                ("\x1b[31;1m", "\x1b[m")
            } else {
                ("", "")
            };

            println!("{}ERROR: {}{}", ansi1, e, ansi2);
            continue;
        }

        let repo_info = res.unwrap();

        if repo_info.unpushed_branches.is_empty() && !repo_info.dirty {
            continue;
        }

        let (ansi1, ansi2) = if enable_ansi_colors {
            ("\x1b[33;1m", "\x1b[m")
        } else {
            ("", "")
        };

        let mut repo_path_str = repo_info.repo_path.display().to_string();
        if repo_path_str.starts_with(&homedir_display) {
            repo_path_str = format!("~{}", &repo_path_str[homedir_display.len()..]);
        }
        println!("{}{}{}", ansi1, repo_path_str, ansi2);

        if !repo_info.unpushed_branches.is_empty() {
            let mut cmd = Command::new("git");

            cmd.arg("-C")
                .arg(repo_path)
                .arg("branch")
                .arg("-vv")
                .arg("--list")
                .stdout(Stdio::inherit());

            for branch in &repo_info.unpushed_branches {
                cmd.arg(branch);
            }

            let mut child = cmd.spawn()?;
            child.wait()?;
        }

        if repo_info.dirty {
            let mut child = Command::new("git")
                .arg("-C")
                .arg(repo_path)
                .arg("status")
                .arg("--short")
                .stdout(Stdio::inherit())
                .spawn()?;
            child.wait()?;
        }
    }

    Ok(())
}

struct RepoInfo {
    repo_path: PathBuf,
    dirty: bool,
    unpushed_branches: Vec<String>,
}

fn process_repo(repo_path: &PathBuf) -> Res<RepoInfo> {
    let repo = Repository::discover(repo_path).map_err(|e| match e.code() {
        ErrorCode::NotFound => anyhow::anyhow!("not a Git repo: {}", repo_path.display()),
        _ => anyhow::Error::from(e),
    })?;

    let statuses = repo.statuses(Some(StatusOptions::new().include_untracked(true)))?;

    let mut repo_info = RepoInfo {
        repo_path: repo_path.to_path_buf(),
        dirty: !statuses.is_empty(),
        unpushed_branches: Vec::new(),
    };

    let mut remote_sha1s = HashSet::new();
    let remote_branches = repo.branches(Some(BranchType::Remote))?;
    for res in remote_branches {
        let (branch, _) = res?;
        let sha1 = branch.get().peel_to_commit()?.id().to_string();
        remote_sha1s.insert(sha1);
    }

    let local_branches = repo.branches(Some(BranchType::Local))?;
    for res in local_branches {
        let (branch, _) = res?;
        let branch_name = branch_to_string(&branch)?;
        let sha1 = branch.get().peel_to_commit()?.id().to_string();

        if !remote_sha1s.contains(sha1.as_str()) {
            repo_info.unpushed_branches.push(branch_name.to_string());
        }
    }

    Ok(repo_info)
}

fn branch_to_string(branch: &Branch) -> Res<String> {
    return Ok(branch
        .name()?
        .ok_or_else(|| anyhow::anyhow!("branch name can't become str"))?
        .to_string());
}
