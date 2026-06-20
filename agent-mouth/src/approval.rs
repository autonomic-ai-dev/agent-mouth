//! Slack approval webhook with agent-heart-compatible bash AST validation.

use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::Path;
use tree_sitter::Node;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub approved: bool,
    pub command: String,
    pub issues: Vec<ValidationIssue>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub line: usize,
    pub severity: String,
    pub message: String,
}

pub fn validate_command(source: &str) -> ValidationReport {
    let mut issues = Vec::new();
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&tree_sitter_bash::LANGUAGE.into()).is_ok() {
        if let Some(tree) = parser.parse(source, None) {
            walk_node(source, tree.root_node(), &mut issues);
        } else {
            issues.push(ValidationIssue {
                line: 1,
                severity: "error".into(),
                message: "failed to parse command as bash".into(),
            });
        }
    }

    let has_error = issues.iter().any(|i| i.severity == "error");
    let has_destructive = issues.iter().any(|i| {
        i.message.contains("rm -rf") || i.message.contains("code injection")
    });

    let approved = !has_error && !has_destructive;
    let reason = if approved {
        if issues.is_empty() {
            "command passed AST validation".into()
        } else {
            format!("approved with {} warning(s)", issues.len())
        }
    } else if has_destructive {
        "rejected: destructive or dynamic execution pattern detected".into()
    } else {
        "rejected: AST validation errors".into()
    };

    ValidationReport {
        approved,
        command: source.to_string(),
        issues,
        reason,
    }
}

pub fn validate_script(path: &Path) -> Result<ValidationReport> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("read script {}", path.display()))?;
    Ok(validate_command(&source))
}

pub fn verify_slack_signature(
    signing_secret: &str,
    timestamp: &str,
    body: &[u8],
    signature: &str,
) -> bool {
    if signing_secret.is_empty() {
        return false;
    }
    let base = format!("v0:{timestamp}:{}", String::from_utf8_lossy(body));
    let mut mac = match HmacSha256::new_from_slice(signing_secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(base.as_bytes());
    let expected = format!("v0={}", hex::encode(mac.finalize().into_bytes()));
    constant_time_eq(signature.as_bytes(), expected.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn walk_node(source: &str, node: Node, issues: &mut Vec<ValidationIssue>) {
    let kind = node.kind();
    let line = node.start_position().row + 1;
    let text = &source[node.byte_range()];

    if kind == "command" {
        let lower = text.to_lowercase();
        if lower.starts_with("rm ")
            && (lower.contains(" -rf ") || lower.contains(" -fr "))
        {
            issues.push(ValidationIssue {
                line,
                severity: "error".into(),
                message: "destructive rm -rf blocked by approval policy".into(),
            });
        }
        if lower.starts_with("eval ")
            || lower.starts_with("source ")
            || lower.starts_with(". ")
        {
            issues.push(ValidationIssue {
                line,
                severity: "error".into(),
                message: "dynamic execution (eval/source/.) blocked".into(),
            });
        }
        if lower.contains("curl ") && lower.contains("| sh") {
            issues.push(ValidationIssue {
                line,
                severity: "error".into(),
                message: "pipe-to-shell pattern blocked".into(),
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_node(source, child, issues);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_rm_rf() {
        let report = validate_command("rm -rf /tmp/foo");
        assert!(!report.approved);
    }

    #[test]
    fn approves_echo() {
        let report = validate_command("echo hello");
        assert!(report.approved);
    }
}
