extern crate git2;
extern crate url;

use std::env;
use std::io;
use std::process;
use std::error;
use std::fmt;

use url::Url;

enum PushType {
    Delete,
    Create,
    Update
}

struct CommitRange {
    local_sha1: git2::Oid,
    remote_sha1: git2::Oid,
}

impl CommitRange {
    fn from_line(line: String) -> Result<Self, HookError> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let local_sha1 = try!(git2::Oid::from_str(parts[1]));
        let remote_sha1 = try!(git2::Oid::from_str(parts[3]));

        return Ok(CommitRange {
            local_sha1: local_sha1,
            remote_sha1: remote_sha1,
        });
    }

    fn range(&self) -> Option<String> {
        match self.push_type() {
            PushType::Delete => None,
            PushType::Create => Some(format!("{}", self.local_sha1)),
            PushType::Update => Some(format!("{}..{}", self.remote_sha1, self.local_sha1)),
        }
    }

    fn push_type(&self) -> PushType {
        if self.local_sha1.is_zero() {
            PushType::Delete
        } else if self.remote_sha1.is_zero() {
            PushType::Create
        } else {
            PushType::Update
        }
    }
}

#[derive(Debug)]
struct Args {
    remote: String,
    url: Url,
}

impl Args {
    fn from_env(mut argv: env::Args) -> Result<Self, String> {
        let remote = try!(argv.nth(1).ok_or("remote arg is required".to_owned()));
        let url = try!(argv.next()
            .ok_or("remote url arg is required".to_owned())
            .and_then(|arg| Url::parse(&arg).map_err(|e| e.to_string()) ));

        return Ok(Args{remote: remote, url: url})
    }
}

#[derive(Debug)]
enum HookError {
    NoRemoteHost,
    Io(io::Error),
    Git(git2::Error),
}

impl fmt::Display for HookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HookError::Io(ref e) => write!(f, "IO Error: {}", e),
            HookError::Git(ref e) => write!(f, "Git Error: {}", e),
            HookError::NoRemoteHost => write!(f, "No Remote Host"),
        }
    }
}

impl error::Error for HookError {
    fn description(&self) -> &str {
        match *self {
            HookError::Io(ref e) => e.description(),
            HookError::Git(ref e) => e.description(),
            HookError::NoRemoteHost => "no remote host",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            HookError::Io(ref e) => Some(e),
            HookError::Git(ref e) => Some(e),
            HookError::NoRemoteHost => None,
        }
    }
}

impl From<io::Error> for HookError {
    fn from(e: io::Error) -> HookError {
        HookError::Io(e)
    }
}

impl From<git2::Error> for HookError {
    fn from(e: git2::Error) -> HookError {
        HookError::Git(e)
    }
}

enum Check {
    Pass,
    Fail(Vec<git2::Oid>),
}

fn enforce_commit_author(args: &Args, commit_info: &CommitRange) -> Result<Check, HookError> {

    match commit_info.push_type() {
        PushType::Delete => return Ok(Check::Pass),
        _ => {}
    }

    let repository_path = try!(env::current_dir());
    let repository = try!(git2::Repository::open(repository_path));

    let host = try!(args.url.host().ok_or(HookError::NoRemoteHost));
    let config_entry = format!("ghostwriter.{}.author", host);

    let git_config = try!(repository.config());
    let author = try!(git_config.get_string(&config_entry));

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
