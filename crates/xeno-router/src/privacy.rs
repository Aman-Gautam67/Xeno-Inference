//! Regex secret & PII scrubber and socket-level air-gap enforcer.

use regex::Regex;
use std::sync::LazyLock;
use url::Url;
use xeno_core::{
    contracts::{ContentBlock, InferenceRequest, PrivacyFilter},
    errors::XenoError,
    types::ProviderKind,
};

/// Pre-compiled redaction rule.
struct RedactionRule {
    name: &'static str,
    pattern: Regex,
    replacement: &'static str,
}

static DEFAULT_RULES: LazyLock<Vec<RedactionRule>> = LazyLock::new(|| {
    vec![
        // AWS Access Key ID
        RedactionRule {
            name: "aws_access_key",
            pattern: Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),
            replacement: "[REDACTED:AWS_ACCESS_KEY]",
        },
        // Anthropic API Key (must precede generic OpenAI sk- prefix)
        RedactionRule {
            name: "anthropic_key",
            pattern: Regex::new(r"\bsk-ant-[a-zA-Z0-9_-]{20,90}\b").unwrap(),
            replacement: "[REDACTED:ANTHROPIC_KEY]",
        },
        // OpenAI API Key
        RedactionRule {
            name: "openai_key",
            pattern: Regex::new(r"\bsk-(?:proj-)?[a-zA-Z0-9_-]{20,100}\b").unwrap(),
            replacement: "[REDACTED:OPENAI_KEY]",
        },
        // GitHub Personal Access Token
        RedactionRule {
            name: "github_pat",
            pattern: Regex::new(r"\b(?:ghp_[a-zA-Z0-9]{36}|github_pat_[a-zA-Z0-9_]{82})\b").unwrap(),
            replacement: "[REDACTED:GITHUB_PAT]",
        },
        // Generic JWT Token
        RedactionRule {
            name: "jwt_token",
            pattern: Regex::new(r"\beyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\b").unwrap(),
            replacement: "[REDACTED:JWT_TOKEN]",
        },
        // Private SSH / RSA / EC Key Blocks
        RedactionRule {
            name: "ssh_private_key",
            pattern: Regex::new(r"-----BEGIN (?:RSA|OPENSSH|EC|DSA|PGP) PRIVATE KEY-----[\s\S]*?-----END (?:RSA|OPENSSH|EC|DSA|PGP) PRIVATE KEY-----").unwrap(),
            replacement: "[REDACTED:PRIVATE_SSH_KEY]",
        },
        // Internal RFC1918 IPv4 addresses
        RedactionRule {
            name: "internal_ip",
            pattern: Regex::new(r"\b(?:10\.\d{1,3}\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2[0-9]|3[0-1])\.\d{1,3}\.\d{1,3})\b").unwrap(),
            replacement: "[REDACTED:INTERNAL_IP]",
        },
    ]
});

/// High-speed pre-flight privacy and secret scrubbing engine.
#[derive(Debug, Clone, Default)]
pub struct PrivacyScrubber;

impl PrivacyScrubber {
    /// Constructs a new [`PrivacyScrubber`].
    pub fn new() -> Self {
        Self
    }

    /// Sanitizes an input string, replacing detected secrets according to the privacy filter.
    pub fn sanitize_text(&self, input: &str, filter: &PrivacyFilter) -> String {
        if !filter.enabled {
            return input.to_string();
        }

        // Strip zero-width and invisible evasion characters before applying regex rules
        let cleaned: String = input
            .chars()
            .filter(|&c| {
                !matches!(
                    c,
                    '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{2060}' | '\u{00AD}' | '\u{200E}' | '\u{200F}'
                )
            })
            .collect();

        let mut output = cleaned;

        for rule in DEFAULT_RULES.iter() {
            if rule.name == "internal_ip" && !filter.redact_pii {
                continue;
            }
            if rule.name != "internal_ip" && !filter.redact_secrets {
                continue;
            }
            output = rule.pattern.replace_all(&output, rule.replacement).into_owned();
        }

        // Apply custom user-defined regex redaction rules
        for custom_pat in &filter.custom_redaction_patterns {
            if let Ok(re) = Regex::new(custom_pat) {
                output = re.replace_all(&output, "[REDACTED:CUSTOM]").into_owned();
            }
        }

        output
    }

    /// Sanitizes all text blocks in an inference request in-place.
    pub fn sanitize_request(&self, req: &mut InferenceRequest, filter: &PrivacyFilter) {
        if !filter.enabled {
            return;
        }

        if let Some(sys) = &mut req.system_prompt {
            *sys = self.sanitize_text(sys, filter);
        }

        for msg in &mut req.messages {
            for block in &mut msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        *text = self.sanitize_text(text, filter);
                    }
                    ContentBlock::Thinking { reasoning } => {
                        *reasoning = self.sanitize_text(reasoning, filter);
                    }
                    ContentBlock::ToolResult { content, .. } => {
                        *content = self.sanitize_text(content, filter);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Socket and provider air-gap isolation enforcer.
#[derive(Debug, Clone, Default)]
pub struct AirGapEnforcer;

impl AirGapEnforcer {
    /// Constructs a new [`AirGapEnforcer`].
    pub fn new() -> Self {
        Self
    }

    /// Checks if a provider kind is permitted under air-gap constraints.
    pub fn validate_provider_kind(
        &self,
        provider: ProviderKind,
        air_gap_mode: bool,
    ) -> Result<(), XenoError> {
        if air_gap_mode && provider.is_cloud() {
            return Err(XenoError::AirGapViolation {
                mode: "AirGapEnforced".into(),
                target: format!("Cloud provider '{}' blocked under air-gap isolation", provider.as_str()),
            });
        }
        Ok(())
    }

    /// Checks if a target endpoint URL is a local loopback address.
    pub fn validate_endpoint_url(
        &self,
        endpoint_url: &str,
        air_gap_mode: bool,
    ) -> Result<(), XenoError> {
        if !air_gap_mode {
            return Ok(());
        }

        let parsed = Url::parse(endpoint_url).map_err(|e| XenoError::InvalidRequest(
            format!("Invalid URL in air-gap validation: {e}"),
        ))?;

        let host = parsed.host_str().unwrap_or("");
        let is_loopback = host == "localhost"
            || host == "127.0.0.1"
            || host == "::1"
            || host == "[::1]"
            || host == "0.0.0.0";

        if !is_loopback {
            return Err(XenoError::AirGapViolation {
                mode: "AirGapEnforced".into(),
                target: format!("Non-loopback network destination '{host}' blocked under air-gap isolation"),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_scrubber_redaction() {
        let scrubber = PrivacyScrubber::new();
        let filter = PrivacyFilter::default();

        let raw_prompt = "Connect to AWS with AKIA1234567890ABCDEF and OpenAI key sk-1234567890abcdef1234567890.";
        let clean = scrubber.sanitize_text(raw_prompt, &filter);

        assert!(clean.contains("[REDACTED:AWS_ACCESS_KEY]"));
        assert!(clean.contains("[REDACTED:OPENAI_KEY]"));
        assert!(!clean.contains("AKIA1234567890ABCDEF"));
    }

    #[test]
    fn test_privacy_scrubber_zero_width_evasion() {
        let scrubber = PrivacyScrubber::new();
        let filter = PrivacyFilter::default();

        let evasion_prompt = "Secret: AKIA\u{200B}1234567890ABCDEF and sk-proj\u{200C}-1234567890abcdef1234567890.";
        let clean = scrubber.sanitize_text(evasion_prompt, &filter);

        assert!(clean.contains("[REDACTED:AWS_ACCESS_KEY]"));
        assert!(clean.contains("[REDACTED:OPENAI_KEY]"));
        assert!(!clean.contains("AKIA"));
        assert!(!clean.contains("sk-proj"));
    }

    #[test]
    fn test_air_gap_enforcer_provider_validation() {
        let enforcer = AirGapEnforcer::new();

        // Local & Mock are allowed
        assert!(enforcer.validate_provider_kind(ProviderKind::Local, true).is_ok());
        assert!(enforcer.validate_provider_kind(ProviderKind::Mock, true).is_ok());

        // Cloud providers are rejected
        assert!(enforcer.validate_provider_kind(ProviderKind::Anthropic, true).is_err());
        assert!(enforcer.validate_provider_kind(ProviderKind::Openai, true).is_err());
        assert!(enforcer.validate_provider_kind(ProviderKind::Google, true).is_err());
        assert!(enforcer.validate_provider_kind(ProviderKind::Deepseek, true).is_err());
        assert!(enforcer.validate_provider_kind(ProviderKind::Groq, true).is_err());
    }

    #[test]
    fn test_air_gap_enforcer_url_validation() {
        let enforcer = AirGapEnforcer::new();

        assert!(enforcer.validate_endpoint_url("http://localhost:8080/v1", true).is_ok());
        assert!(enforcer.validate_endpoint_url("http://127.0.0.1:11434/v1", true).is_ok());

        assert!(enforcer.validate_endpoint_url("https://api.openai.com/v1", true).is_err());
        assert!(enforcer.validate_endpoint_url("https://api.anthropic.com", true).is_err());
    }
}
