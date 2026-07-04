//! Local SLM inference via llama.cpp (HTTP first, subprocess fallback).
//!
//! Env:
//! - `AGENT_MOUTH_LLAMA_URL` — OpenAI-compatible base (default `http://127.0.0.1:8080`)
//! - `AGENT_MOUTH_LLAMA_BIN` — optional `llama-cli` / `llama-server` binary for subprocess
//! - `AGENT_MOUTH_MODEL` — optional model name/path for HTTP or CLI

use anyhow::{anyhow, Context, Result};
use std::process::Command;
use std::time::Duration;

/// Hard cap for all local/API completions (Phase 4).
pub const MAX_TOKENS: u32 = 150;

const DEFAULT_LLAMA_URL: &str = "http://127.0.0.1:8080";

#[derive(Debug, Clone)]
pub struct LocalInferenceConfig {
    pub base_url: String,
    pub llama_bin: Option<String>,
    pub model: Option<String>,
    pub max_tokens: u32,
    pub timeout: Duration,
}

impl Default for LocalInferenceConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl LocalInferenceConfig {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("AGENT_MOUTH_LLAMA_URL")
                .unwrap_or_else(|_| DEFAULT_LLAMA_URL.into()),
            llama_bin: std::env::var("AGENT_MOUTH_LLAMA_BIN").ok().filter(|s| !s.is_empty()),
            model: std::env::var("AGENT_MOUTH_MODEL").ok().filter(|s| !s.is_empty()),
            max_tokens: MAX_TOKENS,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Complete a prompt with the local SLM. Tries HTTP, then subprocess.
pub fn complete(system: &str, user: &str) -> Result<String> {
    complete_with(&LocalInferenceConfig::from_env(), system, user)
}

pub fn complete_with(cfg: &LocalInferenceConfig, system: &str, user: &str) -> Result<String> {
    let max_tokens = cfg.max_tokens.min(MAX_TOKENS);
    match complete_http(cfg, system, user, max_tokens) {
        Ok(text) => Ok(truncate_tokens_approx(&text, max_tokens)),
        Err(http_err) => {
            if let Some(bin) = cfg.llama_bin.as_deref() {
                let text = complete_subprocess(bin, cfg.model.as_deref(), system, user, max_tokens)
                    .with_context(|| format!("subprocess fallback after HTTP error: {http_err}"))?;
                Ok(truncate_tokens_approx(&text, max_tokens))
            } else {
                Err(http_err)
            }
        }
    }
}

/// True when a local backend is configured (URL always is; bin is optional).
pub fn local_backend_available(cfg: &LocalInferenceConfig) -> bool {
    !cfg.base_url.is_empty() || cfg.llama_bin.is_some()
}

fn complete_http(
    cfg: &LocalInferenceConfig,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(cfg.timeout)
        .build()?;

    let base = cfg.base_url.trim_end_matches('/');
    let chat_url = format!("{base}/v1/chat/completions");

    let mut body = serde_json::json!({
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "max_tokens": max_tokens,
        "temperature": 0.2,
    });
    if let Some(model) = &cfg.model {
        body["model"] = serde_json::Value::String(model.clone());
    }

    let resp = client.post(&chat_url).json(&body).send();
    match resp {
        Ok(r) if r.status().is_success() => {
            let json: serde_json::Value = r.json()?;
            if let Some(text) = json["choices"][0]["message"]["content"].as_str() {
                return Ok(text.trim().to_string());
            }
            if let Some(text) = json["choices"][0]["text"].as_str() {
                return Ok(text.trim().to_string());
            }
            Err(anyhow!("llama.cpp chat response missing content: {json}"))
        }
        Ok(r) => {
            // Fall back to native /completion endpoint.
            let status = r.status();
            let _ = r.text();
            complete_http_native(cfg, &client, system, user, max_tokens)
                .with_context(|| format!("chat endpoint status {status}"))
        }
        Err(e) => complete_http_native(cfg, &client, system, user, max_tokens)
            .with_context(|| format!("chat endpoint error: {e}")),
    }
}

fn complete_http_native(
    cfg: &LocalInferenceConfig,
    client: &reqwest::blocking::Client,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<String> {
    let base = cfg.base_url.trim_end_matches('/');
    let url = format!("{base}/completion");
    let prompt = format!("{system}\n\n{user}\n\nSummary:");
    let body = serde_json::json!({
        "prompt": prompt,
        "n_predict": max_tokens,
        "temperature": 0.2,
    });
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .with_context(|| format!("POST {url}"))?;
    if !resp.status().is_success() {
        return Err(anyhow!("completion endpoint status {}", resp.status()));
    }
    let json: serde_json::Value = resp.json()?;
    if let Some(text) = json["content"].as_str() {
        return Ok(text.trim().to_string());
    }
    Err(anyhow!("llama.cpp completion response missing content: {json}"))
}

fn complete_subprocess(
    bin: &str,
    model: Option<&str>,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<String> {
    let prompt = format!("{system}\n\n{user}\n\nSummary:");
    let mut cmd = Command::new(bin);
    if let Some(m) = model {
        cmd.args(["-m", m]);
    }
    cmd.args([
        "-p",
        &prompt,
        "-n",
        &max_tokens.to_string(),
        "--no-display-prompt",
    ]);
    let output = cmd
        .output()
        .with_context(|| format!("spawn {bin}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("{bin} exited {}: {stderr}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Approximate token cap by whitespace words (local models vary).
fn truncate_tokens_approx(text: &str, max_tokens: u32) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= max_tokens as usize {
        return text.trim().to_string();
    }
    words[..max_tokens as usize].join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_tokens_cap_is_150() {
        assert_eq!(MAX_TOKENS, 150);
    }

    #[test]
    fn truncate_respects_cap() {
        let long = (0..300).map(|i| format!("w{i}")).collect::<Vec<_>>().join(" ");
        let out = truncate_tokens_approx(&long, MAX_TOKENS);
        assert_eq!(out.split_whitespace().count(), MAX_TOKENS as usize);
    }

    #[test]
    fn summarize_without_api_key() {
        // No API keys, no live llama server required: heuristics path via summarize.
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("ANTHROPIC_API_KEY");
        // Point HTTP at a closed port so local SLM fails fast into heuristics.
        std::env::set_var("AGENT_MOUTH_LLAMA_URL", "http://127.0.0.1:9");
        std::env::remove_var("AGENT_MOUTH_LLAMA_BIN");

        let input = "ERROR: build failed\nWARN: deprecated API\nINFO: done";
        let out = crate::summarize::summarize_to_string(input).expect("summarize");
        assert!(
            out.contains("Errors") || out.contains("error") || out.contains("Log Summary"),
            "expected heuristic summary, got: {out}"
        );
    }
}
