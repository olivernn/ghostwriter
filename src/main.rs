extern crate git2;
extern crate url;

pub mod commit_range;
pub mod result;
pub mod args;

use commit_range::{PushType, CommitRange};
use result::{HookError, Check};
use args::Args;

use std::env;
use std::io;
use std::process;

fn enforce_commit_author(args: &Args, commit_info: &CommitRange) -> Result<Check, HookError> {

    // nothing to do if this is a delete
    match commit_info.push_type() {
        PushType::Delete => return Ok(Check::Pass),
        _ => {}
    }

    let repository_path = try!(env::current_dir());
    let repository = try!(git2::Repository::open(repository_path));

    let host = try!(args.url.host().ok_or(HookError::NoRemoteHost));
    let config_entry = format!("ghostwriter.{}.author", host);

    let git_config = try!(repository.config());

    // TODO: this is annoying, the git library does not distinguish
    // between an error when getting a config value, and the config
    // value not existing. We assume here that
    let author = match git_config.get_string(&config_entry) {
        Ok(author) => author,
        Err(_) => return Ok(Check::Pass),
    };

    let mut walker = try!(repository.revwalk());

    match commit_info.push_type() {
        PushType::Update => {
            try!(walker.push_range(&commit_info.range().unwrap()));
        },
        PushType::Create => {
            try!(walker.push(commit_info.local_sha1));
        },
        PushType::Delete => {
            panic!("This should never happen because we return early for PushType::Delete");
        }
    }

    let commits_with_wrong_author: Vec<git2::Oid> = walker
        .filter_map(|oid| repository.find_commit(oid).ok() )
        .filter(|commit| {
            commit.author().email().map_or(false, |email| email != &author)
        })
        .map(|commit| commit.id() )
        .collect();

    if commits_with_wrong_author.is_empty() {
        Ok(Check::Pass)
    } else {
        Ok(Check::Fail(commits_with_wrong_author))
    }
}

fn main() {
    let args = Args::from_env(env::args()).unwrap();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Unable to read from stdin");
    let commit_info = CommitRange::from_line(buffer).expect("Unable to parse commit info");

    match enforce_commit_author(&args, &commit_info) {
        Ok(Check::Fail(commit_ids)) => {
            println!("ghostwriter rejecting push due to commits with wrong author:");
            for commit_id in commit_ids {
                println!("{}", commit_id);
            }
            process::exit(1);
        },

        Ok(Check::Pass) => {
            // nothing to do let the program end succesfully
        },

        Err(e) => {
            println!("Error computing check: {}", e);
            process::exit(2);
        }
    };
}
