extern crate git2;

use result::HookError;

pub enum PushType {
    Delete,
    Create,
    Update
}

pub struct CommitRange {
    pub local_sha1: git2::Oid,
    pub remote_sha1: git2::Oid,
}

impl CommitRange {
    pub fn from_line(line: String) -> Result<Self, HookError> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let local_sha1 = try!(git2::Oid::from_str(parts[1]));
        let remote_sha1 = try!(git2::Oid::from_str(parts[3]));

        return Ok(CommitRange {
            local_sha1: local_sha1,
            remote_sha1: remote_sha1,
        });
    }

    pub fn range(&self) -> Option<String> {
        match self.push_type() {
            PushType::Delete => None,
            PushType::Create => Some(format!("{}", self.local_sha1)),
            PushType::Update => Some(format!("{}..{}", self.remote_sha1, self.local_sha1)),
        }
    }

    pub fn push_type(&self) -> PushType {
        if self.local_sha1.is_zero() {
            PushType::Delete
        } else if self.remote_sha1.is_zero() {
            PushType::Create
        } else {
            PushType::Update
        }
    }
}
