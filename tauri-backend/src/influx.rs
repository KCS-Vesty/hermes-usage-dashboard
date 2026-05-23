// tauri-backend/src/influx.rs
// InfluxDB query and CSV parsing utilities.

use serde_json::Value;

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct InfluxConfig {
    pub url: String,
    pub org: String,
    pub bucket: String,
    pub token: String,
}

/// Query InfluxDB for usage data from the last 24 hours, grouped by provider.
pub async fn query_influxdb(config: &InfluxConfig) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let query_url = format!("{}/api/v2/query", config.url);

    let flux_query = format!(
        r#"from(bucket: "{}")
        |> range(start: -24h)
        |> filter(fn: (r) => r._measurement == "usage")
        |> pivot(rowKey:["_time"], columnKey: ["_field"], valueColumn: "_value")
        |> group(columns: ["provider"])
        |> sum()"#,
        config.bucket
    );

    let resp = client
        .post(&query_url)
        .header("Authorization", format!("Token {}", config.token))
        .header("Content-Type", "application/vnd.flux")
        .json(&serde_json::json!({ "query": flux_query, "type": "flux" }))
        .send()
        .await
        .map_err(|e| format!("InfluxDB query failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("InfluxDB returned status {}", resp.status()));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    parse_influx_csv(&text)
}

/// Parse InfluxDB's annotated CSV response into a JSON value with providers array.
///
/// InfluxDB returns CSV with:
/// - Comment lines starting with `#`
/// - A header line with column names (e.g. `result,table,provider,tokens_used,cost_usd`)
/// - Data rows with comma-separated values
pub fn parse_influx_csv(text: &str) -> Result<Value, String> {
    let mut providers = Vec::new();
    let mut total_tokens: i64 = 0;
    let mut total_cost: f64 = 0.0;
    let mut header_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let cols: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        // Header line: map column names to indices
        if header_map.is_empty() {
            for (i, col) in cols.iter().enumerate() {
                header_map.insert(col.to_string(), i);
            }
            continue;
        }

        // Data row
        let get_str = |name: &str| -> &str {
            header_map
                .get(name)
                .and_then(|&i| cols.get(i))
                .copied()
                .unwrap_or("")
        };
        let get_i64 = |name: &str| -> i64 {
            header_map
                .get(name)
                .and_then(|&i| cols.get(i))
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
        };
        let get_f64 = |name: &str| -> f64 {
            header_map
                .get(name)
                .and_then(|&i| cols.get(i))
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0)
        };

        let provider_name = get_str("provider");
        let tokens = get_i64("tokens_used");
        let cost = get_f64("cost_usd");

        if !provider_name.is_empty() && tokens > 0 {
            providers.push(serde_json::json!({
                "name": provider_name,
                "tokens_used": tokens,
                "cost_usd": cost
            }));
            total_tokens += tokens;
            total_cost += cost;
        }
    }

    Ok(serde_json::json!({
        "providers": providers,
        "total_tokens": total_tokens,
        "total_cost_usd": (total_cost * 100.0).round() / 100.0
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_influx_csv_basic() {
        let csv = r#"
#datatype,string,long,string,long,double
#group,false,false,false,false,false
#default,_result,,,,
,result,table,provider,tokens_used,cost_usd
,,0,openrouter,15000,0.45
,,0,anthropic,8000,0.24
"#;

        let result = parse_influx_csv(csv).unwrap();
        let providers = result["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0]["name"], "openrouter");
        assert_eq!(providers[0]["tokens_used"], 15000);
        assert_eq!(providers[0]["cost_usd"], 0.45);
        assert_eq!(result["total_tokens"], 23000);
        assert!((result["total_cost_usd"].as_f64().unwrap() - 0.69).abs() < 0.001);
    }

    #[test]
    fn test_parse_influx_csv_empty() {
        let csv = r#"
#datatype,string,long,string,long,double
#group,false,false,false,false,false
,result,table,provider,tokens_used,cost_usd
"#;

        let result = parse_influx_csv(csv).unwrap();
        let providers = result["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 0);
        assert_eq!(result["total_tokens"], 0);
    }

    #[test]
    fn test_parse_influx_csv_skips_zero_tokens() {
        let csv = r#"
,result,table,provider,tokens_used,cost_usd
,,0,openrouter,15000,0.45
,,0,bad_provider,0,0.0
"#;

        let result = parse_influx_csv(csv).unwrap();
        let providers = result["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0]["name"], "openrouter");
    }

    #[test]
    fn test_parse_influx_csv_all_five_providers() {
        let csv = "result,table,provider,tokens_used,cost_usd
0,0,openrouter,15000,0.45
0,0,anthropic,8000,0.24
0,0,openai,12000,0.36
0,0,opencode zen,5000,0.15
0,0,opencode go,3000,0.09";

        let result = parse_influx_csv(csv).unwrap();
        let providers = result["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 5);
        assert_eq!(result["total_tokens"], 43000);
        assert!((result["total_cost_usd"].as_f64().unwrap() - 1.29).abs() < 0.001);
    }
}
