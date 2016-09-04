extern crate git2;
extern crate url;

use std::env;
use std::io;
use std::process;
use std::error;
use std::fmt;

use url::Url;

struct CommitInfo<'a> {
    local_ref: &'a str,
    local_sha1: &'a str,
    remote_ref: &'a str,
    remote_sha1: &'a str,
}

impl<'a> CommitInfo<'a> {
    fn from_line(line: &'a String) -> Self {
        let parts: Vec<&'a str> = line.split_whitespace().collect();

        return CommitInfo {
            local_ref: parts[0],
            local_sha1: parts[1],
            remote_ref: parts[2],
            remote_sha1: parts[3],
        };
    }

    fn range(&self) -> String {
        format!("{}..{}", self.remote_sha1, self.local_sha1)
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

fn enforce_commit_author(args: &Args, commit_info: &CommitInfo) -> Result<bool, HookError> {
    let repository_path = try!(env::current_dir());
    let repository = try!(git2::Repository::open(repository_path));

    let host = try!(args.url.host().ok_or(HookError::NoRemoteHost));
    let config_entry = format!("ghostwriter.{}.author", host);

    let git_config = try!(repository.config());
    let author = try!(git_config.get_string(&config_entry));

    let mut walker = try!(repository.revwalk());
    try!(walker.push_range(&commit_info.range()));

    let ok = walker
        .map(|oid| repository.find_commit(oid) )
        .filter_map(|commit| commit.ok() )
        .all(|commit| {
            commit.author().email().unwrap() == &author
        });

    return Ok(ok);
}

fn main() {
    let args = Args::from_env(env::args()).unwrap();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Unable to read from stdin");
    let commit_info = CommitInfo::from_line(&buffer);

    match enforce_commit_author(&args, &commit_info) {
        Ok(all_commits_pass) => {
            if !all_commits_pass {
                process::exit(1);
            }
        },

        Err(_) => {
            process::exit(2);
        }
    };

    process::exit(1);
}
