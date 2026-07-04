use anyhow::Result;
use std::io::Read;

use crate::local_inference::{self, LocalInferenceConfig, MAX_TOKENS};

const SYSTEM_PROMPT: &str = "You are a log analyzer. Read the following log output and produce a concise summary with bullet points of errors, warnings, and key events. Focus on actionable issues.";

/// Public entry: stdin → stdout summary.
pub fn summarize() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        println!("No input received. Pipe log content: cat logs.txt | agent-mouth summarize");
        return Ok(());
    }

    let out = summarize_to_string(&input)?;
    println!("{out}");
    Ok(())
}

/// Summarize `input` to a string (testable).
/// Order: template fast path → local SLM → cloud API → heuristics.
pub fn summarize_to_string(input: &str) -> Result<String> {
    if let Some(templated) = template_fast_path(input) {
        return Ok(templated);
    }

    let cfg = LocalInferenceConfig::from_env();
    if local_inference::local_backend_available(&cfg) {
        match local_inference::complete_with(&cfg, SYSTEM_PROMPT, &truncate_input(input)) {
            Ok(text) if !text.is_empty() => return Ok(text),
            Ok(_) => {}
            Err(err) => {
                tracing::debug!(error = %err, "local SLM unavailable, trying API/heuristics");
            }
        }
    }

    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        if let Ok(text) = summarize_with_api(input, &api_key) {
            return Ok(text);
        }
    }
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        if let Ok(text) = summarize_with_anthropic(input, &api_key) {
            return Ok(text);
        }
    }

    Ok(summarize_heuristics(input))
}

/// Structured templates for CI logs and approval-gate payloads (no model).
fn template_fast_path(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    // Approval gate: JSON-ish or key lines.
    if lower.contains("approval")
        && (lower.contains("pending") || lower.contains("approve") || lower.contains("reject"))
    {
        let action = input
            .lines()
            .find(|l| {
                let l = l.to_lowercase();
                l.contains("action") || l.contains("command") || l.contains("node")
            })
            .unwrap_or("(see payload)")
            .trim();
        return Some(format!(
            "Approval Gate\n{}\n  Status: pending human decision\n  Detail: {}\n  Cap: max_tokens={MAX_TOKENS}",
            "━".repeat(14),
            truncate_line(action, 120)
        ));
    }

    // CI summary: test/build markers.
    let has_ci = lower.contains("test result:")
        || lower.contains("cargo test")
        || lower.contains("passed") && lower.contains("failed")
        || lower.contains("##[error]")
        || lower.contains("ci pipeline");
    if has_ci {
        let errors = count_keyword(input, "error");
        let fails = count_keyword(input, "fail");
        let warnings = count_keyword(input, "warn");
        let passed = input
            .lines()
            .any(|l| l.to_lowercase().contains("test result: ok") || l.contains("0 failed"));
        let status = if errors + fails > 0 && !passed {
            "FAILED"
        } else if passed {
            "PASSED"
        } else {
            "UNKNOWN"
        };
        let highlights: Vec<String> = input
            .lines()
            .filter(|l| {
                let l = l.to_lowercase();
                l.contains("error") || l.contains("fail") || l.contains("panic")
            })
            .take(5)
            .map(|l| format!("  - {}", truncate_line(l.trim(), 100)))
            .collect();
        let mut out = format!(
            "CI Summary\n{}\n  Status:   {status}\n  Errors:   {errors}\n  Failures: {fails}\n  Warnings: {warnings}\n",
            "━".repeat(14),
        );
        if !highlights.is_empty() {
            out.push_str("Highlights:\n");
            out.push_str(&highlights.join("\n"));
            out.push('\n');
        }
        return Some(out);
    }

    None
}

fn summarize_heuristics(input: &str) -> String {
    let line_count = input.lines().count();
    let error_count = count_keyword(input, "error");
    let warn_count = count_keyword(input, "warn");
    let fail_count = count_keyword(input, "fail");

    let error_lines: Vec<&str> = input
        .lines()
        .filter(|l| {
            let lower = l.to_lowercase();
            lower.contains("error") || lower.contains("fail") || lower.contains("panic")
        })
        .take(20)
        .collect();

    let mut out = String::new();
    out.push_str("Log Summary\n");
    out.push_str(&"━".repeat(14));
    out.push('\n');
    out.push_str(&format!("  Total lines: {line_count}\n"));
    out.push_str(&format!("  Errors:      {error_count}\n"));
    out.push_str(&format!("  Warnings:    {warn_count}\n"));
    out.push_str(&format!("  Failures:    {fail_count}\n"));
    if !error_lines.is_empty() {
        out.push_str("Key issues:\n");
        for (i, line) in error_lines.iter().enumerate() {
            out.push_str(&format!("  {}. {}\n", i + 1, truncate_line(line.trim(), 117)));
        }
    } else {
        out.push_str("No errors or failures detected.\n");
    }
    out
}

fn summarize_with_api(input: &str, api_key: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": truncate_input(input)}
        ],
        "max_tokens": MAX_TOKENS,
        "temperature": 0.3,
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()?;

    let json: serde_json::Value = resp.json()?;
    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("OpenAI response missing content"))
}

fn summarize_with_anthropic(input: &str, api_key: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": MAX_TOKENS,
        "messages": [
            {"role": "user", "content": format!("{SYSTEM_PROMPT}\n\n{}", truncate_input(input))}
        ],
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()?;

    let json: serde_json::Value = resp.json()?;
    json["content"][0]["text"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("Anthropic response missing content"))
}

fn truncate_input(input: &str) -> String {
    if input.len() > 8000 {
        format!("{}... [truncated from {} chars]", &input[..8000], input.len())
    } else {
        input.to_string()
    }
}

fn truncate_line(line: &str, max: usize) -> String {
    if line.len() > max {
        format!("{}...", &line[..max.saturating_sub(3)])
    } else {
        line.to_string()
    }
}

fn count_keyword(input: &str, keyword: &str) -> usize {
    input
        .lines()
        .filter(|l| l.to_lowercase().contains(keyword))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_heuristics_with_errors() {
        let input = "INFO: starting server\nERROR: connection refused\nWARN: retry attempt 1\nERROR: timeout after 30s\nINFO: shutdown complete";
        let result = summarize_heuristics(input);
        assert!(result.contains("Errors:"));
        assert!(result.contains("2") || result.contains("Errors:      2"));
    }

    #[test]
    fn template_ci_fast_path() {
        let input = "running tests\ntest result: FAILED. 1 passed; 2 failed\nERROR: boom\n##[error] Process completed with exit code 1";
        let out = template_fast_path(input).expect("ci template");
        assert!(out.contains("CI Summary"));
        assert!(out.contains("FAILED") || out.contains("Errors"));
    }

    #[test]
    fn template_approval_fast_path() {
        let input = "approval pending\naction: deploy production\nnode: release_gate";
        let out = template_fast_path(input).expect("approval template");
        assert!(out.contains("Approval Gate"));
    }

    #[test]
    fn summarize_without_api_key_uses_heuristics_or_template() {
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::set_var("AGENT_MOUTH_LLAMA_URL", "http://127.0.0.1:9");
        std::env::remove_var("AGENT_MOUTH_LLAMA_BIN");
        let out = summarize_to_string("ERROR: x\nWARN: y").unwrap();
        assert!(!out.is_empty());
    }
}
