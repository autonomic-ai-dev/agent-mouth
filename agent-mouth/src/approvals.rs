//! File-backed approval queue. A request waits until someone runs `approve` or `deny`, or until it times out.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Approved,
    Denied,
    Expired,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Approval {
    pub id: String,
    pub summary: String,
    pub status: Status,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Approved,
    Denied,
    TimedOut,
}

fn approval_path(dir: &Path, id: &str) -> Result<PathBuf> {
    let valid = !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid {
        bail!("invalid approval id `{id}`: use letters, digits, '-' or '_'");
    }
    Ok(dir.join(format!("{id}.json")))
}

fn read_approval(path: &Path) -> Result<Approval> {
    let bytes =
        std::fs::read(path).with_context(|| format!("no approval at {}", path.display()))?;
    serde_json::from_slice(&bytes).context("approval file is corrupt")
}

pub fn open(dir: &Path, id: &str, summary: &str) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let approval = Approval {
        id: id.to_string(),
        summary: summary.to_string(),
        status: Status::Pending,
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(approval_path(dir, id)?)
        .with_context(|| format!("approval {id} already exists"))?;
    file.write_all(&serde_json::to_vec_pretty(&approval)?)?;
    Ok(())
}

pub fn decide(dir: &Path, id: &str, status: Status) -> Result<()> {
    let path = approval_path(dir, id)?;
    let mut approval = read_approval(&path)?;
    if approval.status != Status::Pending {
        bail!("approval {id} is already {:?}", approval.status);
    }
    approval.status = status;
    std::fs::write(&path, serde_json::to_vec_pretty(&approval)?)?;
    Ok(())
}

pub fn wait(dir: &Path, id: &str, timeout: Duration, poll: Duration) -> Result<Decision> {
    let path = approval_path(dir, id)?;
    let deadline = Instant::now() + timeout;
    loop {
        match read_approval(&path)?.status {
            Status::Approved => return Ok(Decision::Approved),
            Status::Denied => return Ok(Decision::Denied),
            Status::Expired => return Ok(Decision::TimedOut),
            Status::Pending => {}
        }
        if Instant::now() >= deadline {
            let mut approval = read_approval(&path)?;
            if approval.status == Status::Pending {
                approval.status = Status::Expired;
                std::fs::write(&path, serde_json::to_vec_pretty(&approval)?)?;
            }
            return Ok(Decision::TimedOut);
        }
        std::thread::sleep(poll);
    }
}

pub fn list_pending(dir: &Path) -> Result<Vec<Approval>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut pending = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let approval = read_approval(&path)?;
            if approval.status == Status::Pending {
                pending.push(approval);
            }
        }
    }
    pending.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(pending)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("mouth-approvals-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn approved_request_returns_approved() {
        let dir = fresh_dir("approve");
        open(&dir, "scan-1", "lodash HIGH").unwrap();
        decide(&dir, "scan-1", Status::Approved).unwrap();
        let outcome = wait(
            &dir,
            "scan-1",
            Duration::from_millis(50),
            Duration::from_millis(5),
        )
        .unwrap();
        assert_eq!(outcome, Decision::Approved);
    }

    #[test]
    fn denied_request_returns_denied() {
        let dir = fresh_dir("deny");
        open(&dir, "scan-2", "lodash HIGH").unwrap();
        decide(&dir, "scan-2", Status::Denied).unwrap();
        let outcome = wait(
            &dir,
            "scan-2",
            Duration::from_millis(50),
            Duration::from_millis(5),
        )
        .unwrap();
        assert_eq!(outcome, Decision::Denied);
    }

    #[test]
    fn undecided_request_times_out_instead_of_approving() {
        let dir = fresh_dir("timeout");
        open(&dir, "scan-3", "lodash HIGH").unwrap();
        let outcome = wait(
            &dir,
            "scan-3",
            Duration::from_millis(30),
            Duration::from_millis(5),
        )
        .unwrap();
        assert_eq!(outcome, Decision::TimedOut);
        assert!(list_pending(&dir).unwrap().is_empty());
        assert!(decide(&dir, "scan-3", Status::Approved).is_err());
    }

    #[test]
    fn decision_cannot_be_changed_once_made() {
        let dir = fresh_dir("twice");
        open(&dir, "scan-4", "lodash HIGH").unwrap();
        decide(&dir, "scan-4", Status::Approved).unwrap();
        assert!(decide(&dir, "scan-4", Status::Denied).is_err());
    }

    #[test]
    fn duplicate_request_id_is_rejected() {
        let dir = fresh_dir("dup");
        open(&dir, "scan-5", "a").unwrap();
        assert!(open(&dir, "scan-5", "b").is_err());
    }

    #[test]
    fn path_traversal_ids_are_rejected() {
        let dir = fresh_dir("traversal");
        assert!(open(&dir, "../escape", "x").is_err());
        assert!(open(&dir, "", "x").is_err());
    }

    #[test]
    fn list_pending_excludes_decided_requests() {
        let dir = fresh_dir("list");
        open(&dir, "scan-6", "a").unwrap();
        open(&dir, "scan-7", "b").unwrap();
        decide(&dir, "scan-7", Status::Approved).unwrap();
        let pending = list_pending(&dir).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "scan-6");
    }
}
