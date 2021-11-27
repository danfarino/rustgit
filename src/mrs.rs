use glob::glob;
use std::fs;
use std::path::PathBuf;
use git2::{BranchType, ErrorCode, Repository};

type Res<T> = anyhow::Result<T>;

pub fn command_multi_repo_status() -> Res<()> {
    let repo_list = fs::read_to_string("/home/dan/rustgit.rc")?;

    let repo_globs: Vec<&str> = repo_list.split_whitespace().collect();

    println!("{:?}", repo_globs);

    let mut repos = Vec::new();
    for repo_glob in repo_globs {
        let mut matched = false;
        for res in glob(repo_glob)? {
            let path = res?;
            repos.push(path);
            matched = true;
        }

        if !matched {
            repos.push(PathBuf::from(repo_glob));
        }
    }

    repos.sort();

    for repo_path in &repos {
        let repo = Repository::discover(repo_path).map_err(|e| match e.code() {
            ErrorCode::NotFound => anyhow::anyhow!("Please run this from inside of a Git repo"),
            _ => anyhow::Error::from(e),
        })?;

        println!("{}", repo_path.to_str().ok_or(anyhow::Error::msg("repoPath can't become str"))?);

        let branches = repo.branches(Some(BranchType::Local))?;
        for res in branches {
            let (branch,_) = res?;
            let branch_name = branch.name()?.ok_or(anyhow::Error::msg("branch name can't become str"))?;
            println!("  {}", branch_name)
        }
    }

    Ok(())
}
