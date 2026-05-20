# CONTEXT.md — Hermes Usage Dashboard

## Domain Glossary

| Term | Definition |
|------|------------|
| **Usage** | A record of API consumption: which provider, which model, how many tokens, at what cost, at what time. |
| **RateLimit** | A snapshot of rate-limit state for a provider/model: hard limit, soft limit, remaining requests, reset timestamp. |
| **Provider** | An LLM API provider (e.g. openrouter, anthropic, openai). |
| **Model** | A specific model within a provider (e.g. claude-3, gpt-4o). Optional — some usage records are provider-level only. |
| **Metric** | Either a Usage or RateLimit record — the two kinds of data the system collects. |
| **Storage** | The interface behind which metrics are buffered before being written to persistent storage. |
| **Writer** | The module that drains buffered metrics and sends them to InfluxDB. |
| **Dashboard** | The Tauri desktop application that displays usage data. |
| **Command** | A Tauri IPC command exposed to the frontend (e.g. get_usage_summary). |

## Architecture Decisions

- Metrics are buffered in-memory before batch-write to InfluxDB.
- The collector module exposes a `Storage` trait; the default adapter is `InMemoryStorage`.
- The db module provides `format_usage_lines()` and `format_rate_lines()` pure functions for line protocol formatting, and `start_writer()` for the background write loop. Errors are logged via `log::warn!`.
- Domain types (`UsageRecord`, `RateLimitRecord`) are defined once in `monitor::usage` and re-exported from the crate root.
- Tauri command functions are thin wrappers; the plain logic is in separate testable functions (`build_usage_summary()`).
- The `legacy` module provides backward-compatible `record_usage()`/`drain_usage()`/`record_rate()`/`drain_rate()` functions delegating to `InMemoryStorage` behind global statics. Tests use `clear_usage()`/`clear_rate()` for isolation.
