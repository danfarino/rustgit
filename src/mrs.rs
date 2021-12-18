use crate::{Res, Verbosity};
use git2::{Branch, BranchType, ErrorCode, Repository, StatusOptions};
use glob::glob;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn command_multi_repo_status(verbosity: Verbosity) -> Res<()> {
    let enable_ansi_colors = atty::is(atty::Stream::Stdout);

    let homedir = dirs::home_dir().ok_or(anyhow::anyhow!("cannot locate user home dir"))?;
    let config_path = homedir.join(".rustgitrc");
    let config_contents = fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("cannot open {:?}: {}", &config_path, e))?;

    let mut repos_paths = Vec::new();

    for config_line in config_contents.lines() {
        let repo_glob = shellexpand::tilde(config_line);

        let mut matched = false;

        for res in glob(repo_glob.as_ref())? {
            let path = res?;
            repos_paths.push(path);
            matched = true;
        }

        if !matched {
            repos_paths.push(PathBuf::from(repo_glob.as_ref()));
        }
    }

    repos_paths.sort();
    repos_paths.dedup();

    for repo_path in &repos_paths {
        if verbosity >= Verbosity::Info {
            println!("Checking repo: {}", repo_path.display());
        }

        let res = process_repo(repo_path, verbosity);
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

        let mut repo_path_display = repo_info.repo_path;
        if let Ok(stripped) = repo_path.strip_prefix(&homedir) {
            repo_path_display = PathBuf::from("~").join(stripped);
        }
        println!("{}{}{}", ansi1, repo_path_display.display(), ansi2);

        if !repo_info.unpushed_branches.is_empty() {
            let mut cmd = Command::new("git");

            cmd.arg("-C")
                .arg(repo_path)
                .arg("branch")
                .arg("-vv")
                .arg("--list");

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

fn process_repo(repo_path: &Path, verbosity: Verbosity) -> Res<RepoInfo> {
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

    let mut remote_shas = HashMap::new();
    let remote_branches = repo.branches(Some(BranchType::Remote))?;
    for res in remote_branches {
        let (branch, _) = res?;
        let branch_name = branch_to_string(&branch)?;
        let branch_head_sha = branch.get().peel_to_commit()?.id();
        remote_shas.insert(branch_head_sha, branch_name);
    }

    let local_branches = repo.branches(Some(BranchType::Local))?;
    'local_branches: for res in local_branches {
        let (branch, _) = res?;
        let branch_name = branch_to_string(&branch)?;
        let branch_head_sha = branch.get().peel_to_commit()?.id();

        if verbosity >= Verbosity::Debug {
            println!(
                "  considering local branch \"{}\" {}",
                branch_name, branch_head_sha
            )
        }

        if let Some(remote_branch_name) = remote_shas.get(&branch_head_sha) {
            if verbosity >= Verbosity::Debug {
                println!("    matches \"{}\"", remote_branch_name);
            }
            continue;
        }

        if verbosity >= Verbosity::Debug {
            println!("    no remote branch at same commit");
        }

        for (remote_sha, remote_branch_name) in &remote_shas {
            let (ahead, behind) = repo.graph_ahead_behind(branch_head_sha, *remote_sha)?;
            if verbosity >= Verbosity::Debug {
                println!(
                    "    checking {} \"{}\" ahead={} behind={}",
                    remote_sha, remote_branch_name, ahead, behind
                );
            }
            if ahead == 0 {
                if verbosity >= Verbosity::Debug {
                    println!("      found matching commit on {}", remote_branch_name);
                }
                continue 'local_branches;
            }
        }

        repo_info.unpushed_branches.push(branch_name.to_string());
    }

    Ok(repo_info)
}

fn branch_to_string(branch: &Branch) -> Res<String> {
    return Ok(branch
        .name()?
        .ok_or_else(|| anyhow::anyhow!("branch name can't become str"))?
        .to_string());
}
