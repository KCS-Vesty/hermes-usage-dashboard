use crate::collector::{drain_rate, drain_usage};
use crate::usage::{UsageRecord, RateLimitRecord};
use influxdb2::Client;
use tokio::time::{sleep, Duration};

/// Escape special characters in InfluxDB line protocol tag values.
/// Per the InfluxDB spec, spaces, commas, and equals signs must be escaped with a backslash.
fn escape_tag_value(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(' ', "\\ ")
        .replace(',', "\\,")
        .replace('=', "\\=")
}

/// Format a batch of UsageRecords into InfluxDB line protocol.
/// Pure function — no I/O, easily testable.
pub fn format_usage_lines(records: &[UsageRecord]) -> Vec<String> {
    records
        .iter()
        .map(|r| {
            format!(
                "usage,provider={},model={} tokens_used={},cost_usd={} {}",
                escape_tag_value(&r.provider),
                escape_tag_value(r.model.as_deref().unwrap_or("")),
                r.tokens_used,
                r.cost_usd,
                r.ts * 1_000_000_000
            )
        })
        .collect()
}

/// Format a batch of RateLimitRecords into InfluxDB line protocol.
/// Pure function — no I/O, easily testable.
pub fn format_rate_lines(records: &[RateLimitRecord]) -> Vec<String> {
    records
        .iter()
        .map(|r| {
            format!(
                "rate_limit,provider={},model={} hard_limit={},soft_limit={},remaining={},reset_ts={} {}",
                escape_tag_value(&r.provider),
                escape_tag_value(r.model.as_deref().unwrap_or("")),
                r.hard_limit,
                r.soft_limit.unwrap_or(-1),
                r.remaining,
                r.reset_ts,
                r.ts * 1_000_000_000
            )
        })
        .collect()
}

pub async fn start_writer() {
    let client = Client::new(
        "http://127.0.0.1:8086",
        "hermes",
        "hermes-token",
    );
    let bucket = "hermes_usage";
    let org = "hermes";

    loop {
        sleep(Duration::from_secs(2)).await;

        let usage_batch = drain_usage().await;
        if !usage_batch.is_empty() {
            let lines = format_usage_lines(&usage_batch);
            write_batch(&client, org, bucket, lines.join("\n")).await;
        }

        let rate_batch = drain_rate().await;
        if !rate_batch.is_empty() {
            let lines = format_rate_lines(&rate_batch);
            write_batch(&client, org, bucket, lines.join("\n")).await;
        }
    }
}

/// Write a batch of line protocol to InfluxDB, logging errors instead of silently dropping them.
async fn write_batch(client: &Client, org: &str, bucket: &str, body: String) {
    if let Err(e) = client.write_line_protocol(org, bucket, body).await {
        log::warn!("InfluxDB write failed: {}", e);
    }
}

#[cfg(test)]
mod format_tests {
    use crate::usage::{UsageRecord, RateLimitRecord};
    use crate::db::{format_rate_lines, format_usage_lines};

    #[test]
    fn test_format_usage_lines_single() {
        let records = vec![UsageRecord {
            provider: "openrouter".to_string(),
            model: Some("claude-3".to_string()),
            tokens_used: 1000,
            cost_usd: 0.002,
            ts: 1_700_000_000,
        }];
        let lines = format_usage_lines(&records);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("usage,provider=openrouter,model=claude-3"));
        assert!(lines[0].contains("tokens_used=1000"));
        assert!(lines[0].contains("cost_usd=0.002"));
        // timestamp: 1_700_000_000 * 1_000_000_000 = 1700000000000000000
        assert!(lines[0].contains("1700000000000000000"));
    }

    #[test]
    fn test_format_usage_lines_no_model() {
        let records = vec![UsageRecord {
            provider: "anthropic".to_string(),
            model: None,
            tokens_used: 500,
            cost_usd: 0.015,
            ts: 1_700_000_000,
        }];
        let lines = format_usage_lines(&records);
        assert!(lines[0].contains("model="));
        // model should be empty when None
        let model_part = lines[0].split("model=").nth(1).unwrap();
        assert!(model_part.starts_with(" "));
    }

    #[test]
    fn test_format_usage_lines_empty() {
        let records: Vec<UsageRecord> = vec![];
        let lines = format_usage_lines(&records);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_format_usage_lines_multiple() {
        let records = vec![
            UsageRecord {
                provider: "openrouter".to_string(),
                model: None,
                tokens_used: 100,
                cost_usd: 0.001,
                ts: 1_700_000_000,
            },
            UsageRecord {
                provider: "anthropic".to_string(),
                model: None,
                tokens_used: 200,
                cost_usd: 0.002,
                ts: 1_700_000_001,
            },
        ];
        let lines = format_usage_lines(&records);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("provider=openrouter"));
        assert!(lines[1].contains("provider=anthropic"));
    }

    #[test]
    fn test_format_rate_lines_single() {
        let records = vec![RateLimitRecord {
            provider: "openrouter".to_string(),
            model: Some("gpt-4o".to_string()),
            hard_limit: 10000,
            soft_limit: Some(5000),
            remaining: 7500,
            reset_ts: 1_700_000_100,
            ts: 1_700_000_000,
        }];
        let lines = format_rate_lines(&records);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("rate_limit,provider=openrouter,model=gpt-4o"));
        assert!(lines[0].contains("hard_limit=10000"));
        assert!(lines[0].contains("soft_limit=5000"));
        assert!(lines[0].contains("remaining=7500"));
        assert!(lines[0].contains("reset_ts=1700000100"));
    }

    #[test]
    fn test_format_rate_lines_no_soft_limit() {
        let records = vec![RateLimitRecord {
            provider: "openai".to_string(),
            model: None,
            hard_limit: 5000,
            soft_limit: None,
            remaining: 3000,
            reset_ts: 1_700_000_100,
            ts: 1_700_000_000,
        }];
        let lines = format_rate_lines(&records);
        assert!(lines[0].contains("soft_limit=-1"));
    }

    #[test]
    fn test_format_usage_lines_escapes_special_chars() {
        let records = vec![UsageRecord {
            provider: "my provider".to_string(),
            model: Some("claude 3.5".to_string()),
            tokens_used: 1000,
            cost_usd: 0.002,
            ts: 1_700_000_000,
        }];
        let lines = format_usage_lines(&records);
        assert_eq!(lines.len(), 1);
        // Spaces in tag values must be escaped with backslash per InfluxDB line protocol
        assert!(lines[0].contains("provider=my\\ provider"), "provider not escaped: {}", lines[0]);
        assert!(lines[0].contains("model=claude\\ 3.5"), "model not escaped: {}", lines[0]);
    }

    #[test]
    fn test_format_rate_lines_escapes_special_chars() {
        let records = vec![RateLimitRecord {
            provider: "my provider".to_string(),
            model: Some("gpt=4o".to_string()),
            hard_limit: 10000,
            soft_limit: Some(5000),
            remaining: 7500,
            reset_ts: 1_700_000_100,
            ts: 1_700_000_000,
        }];
        let lines = format_rate_lines(&records);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("provider=my\\ provider"), "provider not escaped: {}", lines[0]);
        assert!(lines[0].contains("model=gpt\\=4o"), "model not escaped: {}", lines[0]);
    }

    #[test]
    fn test_format_rate_lines_empty() {
        let records: Vec<RateLimitRecord> = vec![];
        let lines = format_rate_lines(&records);
        assert!(lines.is_empty());
    }
}
