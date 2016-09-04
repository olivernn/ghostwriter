extern crate git2;
extern crate url;

use std::env;
use std::io;
use std::process;
use std::path::Path;

use url::{Url, ParseError};

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

fn main() {
    let mut args = env::args();

    let remote = args.nth(1).unwrap();
    let raw_url = args.next().unwrap();
    let url = Url::parse(&raw_url).unwrap();

    println!("remote: {}", remote);
    println!("url: {}", url.host_str().unwrap());

    let mut input = String::new();
    let commit_info = match io::stdin().read_line(&mut input) {
        Ok(_) => {
            CommitInfo::from_line(&input)
        }
        Err(error) => {
            panic!("unable to read from stdin")
        }
    };

    let repository_path = env::current_dir().unwrap();
    let repository = git2::Repository::open(repository_path).unwrap();

    let config_entry = format!("ghostwriter.{}.author", url.host().unwrap());
    let config = repository.config().unwrap();
    let author = config.get_string(&config_entry).unwrap();

    let mut walker = repository.revwalk().unwrap();
    walker.push_range(&commit_info.range()).unwrap();

    let ok = walker
        .map(|oid| repository.find_commit(oid) )
        .filter_map(|commit| commit.ok() )
        .all(|commit| {
            commit.author().email().unwrap() == author
        });

    println!("{}", commit_info.range());
    println!("{}", author);
    println!("{}", ok);

    process::exit(1);
}
