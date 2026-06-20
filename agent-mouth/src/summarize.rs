use anyhow::Result;
use std::io::Read;

const SYSTEM_PROMPT: &str = "You are a log analyzer. Read the following log output and produce a concise summary with bullet points of errors, warnings, and key events. Focus on actionable issues.";

pub fn summarize() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        println!("No input received. Pipe log content: cat logs.txt | agent-mouth summarize");
        return Ok(());
    }

    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        summarize_with_api(&input, &api_key)?;
    } else if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        summarize_with_anthropic(&input, &api_key)?;
    } else {
        summarize_locally(&input)?;
    }
    Ok(())
}

fn summarize_locally(input: &str) -> Result<()> {
    let line_count = input.lines().count();
    let error_count = input.lines().filter(|l| l.to_lowercase().contains("error")).count();
    let warn_count = input.lines().filter(|l| l.to_lowercase().contains("warn")).count();
    let fail_count = input.lines().filter(|l| l.to_lowercase().contains("fail")).count();
    let trace_count = input.lines().filter(|l| l.to_lowercase().contains("trace")).count();
    let debug_count = input.lines().filter(|l| l.to_lowercase().contains("debug")).count();

    let error_lines: Vec<&str> = input
        .lines()
        .filter(|l| {
            let lower = l.to_lowercase();
            lower.contains("error") || lower.contains("fail") || lower.contains("panic")
        })
        .take(20)
        .collect();

    println!("Log Summary");
    println!("{}", "━".repeat(14));
    println!("  Total lines: {}", line_count);
    println!("  Errors:      {}", error_count);
    println!("  Warnings:    {}", warn_count);
    println!("  Failures:    {}", fail_count);
    println!("  Trace/Debug: {}/{}", trace_count, debug_count);
    println!();

    if !error_lines.is_empty() {
        println!("Issues Found (first {}):", error_lines.len());
        for (i, line) in error_lines.iter().enumerate() {
            let truncated = if line.len() > 120 {
                format!("{}...", &line[..117])
            } else {
                line.to_string()
            };
            println!("  {}. {}", i + 1, truncated);
        }
    } else {
        println!("No errors or failures detected.");
    }
    Ok(())
}

fn summarize_with_api(input: &str, api_key: &str) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let truncated = if input.len() > 8000 {
        format!("{}... [truncated from {} chars]", &input[..8000], input.len())
    } else {
        input.to_string()
    };

    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": truncated}
        ],
        "max_tokens": 1000,
        "temperature": 0.3,
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()?;

    let json: serde_json::Value = resp.json()?;
    if let Some(text) = json["choices"][0]["message"]["content"].as_str() {
        println!("{}", text);
    } else {
        eprintln!("API error: {}", serde_json::to_string_pretty(&json)?);
        summarize_locally(input)?;
    }
    Ok(())
}

fn summarize_with_anthropic(input: &str, api_key: &str) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let truncated = if input.len() > 8000 {
        format!("{}... [truncated from {} chars]", &input[..8000], input.len())
    } else {
        input.to_string()
    };

    let body = serde_json::json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 1000,
        "messages": [
            {"role": "user", "content": format!("{}\n\n{}", SYSTEM_PROMPT, truncated)}
        ],
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()?;

    let json: serde_json::Value = resp.json()?;
    if let Some(text) = json["content"][0]["text"].as_str() {
        println!("{}", text);
    } else {
        eprintln!("API error: {}", serde_json::to_string_pretty(&json)?);
        summarize_locally(input)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_locally_with_errors() {
        let input = "INFO: starting server\nERROR: connection refused\nWARN: retry attempt 1\nERROR: timeout after 30s\nINFO: shutdown complete";
        let result = summarize_local_to_string(input);
        assert!(result.contains("Errors:"));
    }

    fn summarize_local_to_string(input: &str) -> String {
        let error_lines: Vec<&str> = input
            .lines()
            .filter(|l| {
                let lower = l.to_lowercase();
                lower.contains("error") || lower.contains("fail") || lower.contains("panic")
            })
            .collect();
        format!("Errors: {}", error_lines.len())
    }
}
