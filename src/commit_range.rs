extern crate git2;

use result::HookError;

#[derive(Debug, PartialEq)]
pub enum PushType {
    Delete,
    Create,
    Update
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::{CommitRange, PushType};

    #[test]
    fn create_type() {
        let line = "master cc4750c575cbfe9c29103c9b76f4a6202824098c origin 0000000000000000000000000000000000000000".to_owned();
        let commit_range = CommitRange::from_line(line);
        assert!(commit_range.is_ok());

        let commit_range = commit_range.unwrap();
        assert_eq!(commit_range.push_type(), PushType::Create);
        assert!(commit_range.range().is_some());

        let range = commit_range.range().unwrap();
        assert_eq!(range, "cc4750c575cbfe9c29103c9b76f4a6202824098c".to_owned());
    }

    #[test]
    fn delete_type() {
        let line = "master 0000000000000000000000000000000000000000 origin cc4750c575cbfe9c29103c9b76f4a6202824098c".to_owned();
        let commit_range = CommitRange::from_line(line);
        assert!(commit_range.is_ok());

        let commit_range = commit_range.unwrap();
        assert_eq!(commit_range.push_type(), PushType::Delete);
        assert!(commit_range.range().is_none());
    }

    #[test]
    fn update_type() {
        let line = "master da36a2a28d015ec274ba035635024348ced15f71 origin cc4750c575cbfe9c29103c9b76f4a6202824098c".to_owned();
        let commit_range = CommitRange::from_line(line);
        assert!(commit_range.is_ok());

        let commit_range = commit_range.unwrap();
        assert_eq!(commit_range.push_type(), PushType::Update);
        assert!(commit_range.range().is_some());

        let range = commit_range.range().unwrap();
        assert_eq!(range, "cc4750c575cbfe9c29103c9b76f4a6202824098c..da36a2a28d015ec274ba035635024348ced15f71".to_owned());
    }

    #[test]
    fn not_enough_parameters() {
        let line = "too short".to_owned();
        let commit_range = CommitRange::from_line(line);
        assert!(commit_range.is_err());
    }

    #[test]
    fn non_oid_parameters() {
        let line = "master notoid origin notoid".to_owned();
        let commit_range = CommitRange::from_line(line);
        assert!(commit_range.is_err());
    }
}
