# Dan's Rust-based Git utility

## Prerequisites

[Install](https://www.rust-lang.org/learn/get-started) the Rust compiler. As of this writing, Rust `1.56.1` is the current version.

## Installation

`cargo install --path .`

## Usage

### Help

```
$ rustgit --help
rustgit 1.0

Dan Farino

Dan's Rust-based Git utility

USAGE:
    rustgit [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help    Print this message or the help of the given subcommand(s)
    mrs     Multi-repo status
    rb      Shows recently-used branches by looking at the reflog
```

### Show Recent Branches

From any of your Git repositories, run `rustgit rb`.

Example:

```
$ rustgit rb
feature/lasers                        1 day     Sat, 27 Nov 2021 17:25:05 -0800
develop                               1 week    Wed, 17 Nov 2021 14:31:03 -0800
main                                  3 weeks   Wed, 03 Nov 2021 19:44:59 -0700
feature/no-more-crashes               3 weeks   Tue, 02 Nov 2021 14:39:49 -0700
```

This uses the Git reflog to produce its output. Since reflog entries expire after a time, this list may not show older branches.

### Multi-repo status

This command checks for a dirty-working directory and/or unpushed local branches in multiple repos.

First, create a file named `.rustgitrc` in your home directory. This file should contain a list of all of the repos you are interested in checking. You may use shell glob wildcards.

Example:
```
~/dev/github/me/*
~/dev/other
```

Then, run `rustgit mrs` to show the "multi-repo status" for all directories that match any of those wildcards.

This example shows two repos. Both have dirty working directories, and one has a branch named `work1` with commits that have not been pushed to a remote server.
```
$ rustgit mrs
~/dev/github/me/myrepo
?? README.md
~/dev/other
  work1 8e3053f [origin/master: ahead 2, behind 1] experimenting
?? src/main.ts
```

In accordance with the Unix philosophy "*no news is good news*", if there are no unpushed branches or dirty working directories, there will be no output.

If you'd like to see more detailed output, run `rustgit mrs -v` or `rustgit mrs -vv`. 