use crate::collector::{RATE_QUEUE, USAGE_QUEUE};
use influxdb2::Client;
use tokio::time::{sleep, Duration};

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

        let mut usage_batch = Vec::new();
        {
            let mut q = USAGE_QUEUE.lock().await;
            while let Some(rec) = q.pop_front() {
                usage_batch.push(rec);
            }
        }

        if !usage_batch.is_empty() {
            let lines: Vec<String> = usage_batch
                .into_iter()
                .map(|r| {
                    format!(
                        "usage,provider={},model={} tokens_used={},cost_usd={} {}",
                        r.provider,
                        r.model.as_deref().unwrap_or(""),
                        r.tokens_used,
                        r.cost_usd,
                        r.ts * 1_000_000_000
                    )
                })
                .collect();
            let _ = client
                .write_line_protocol(org, bucket, lines.join("\n"))
                .await;
        }

        let mut rate_batch = Vec::new();
        {
            let mut q = RATE_QUEUE.lock().await;
            while let Some(rec) = q.pop_front() {
                rate_batch.push(rec);
            }
        }

        if !rate_batch.is_empty() {
            let lines: Vec<String> = rate_batch
                .into_iter()
                .map(|r| {
                    format!(
                        "rate_limit,provider={},model={} hard_limit={},soft_limit={},remaining={},reset_ts={} {}",
                        r.provider,
                        r.model.as_deref().unwrap_or(""),
                        r.hard_limit,
                        r.soft_limit.unwrap_or(-1),
                        r.remaining,
                        r.reset_ts,
                        r.ts * 1_000_000_000
                    )
                })
                .collect();
            let _ = client
                .write_line_protocol(org, bucket, lines.join("\n"))
                .await;
        }
    }
}
