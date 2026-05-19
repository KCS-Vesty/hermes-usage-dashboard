use crate::usage::{UsageRecord, RateLimitRecord};
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

pub static USAGE_QUEUE: Lazy<Arc<Mutex<VecDeque<UsageRecord>>>> =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::with_capacity(1024))));
pub static RATE_QUEUE: Lazy<Arc<Mutex<VecDeque<RateLimitRecord>>>> =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::with_capacity(1024))));

pub async fn record_usage(rec: UsageRecord) {
    let mut q = USAGE_QUEUE.lock().await;
    if q.len() == q.capacity() { q.pop_front(); }
    q.push_back(rec);
}

pub async fn record_rate(rec: RateLimitRecord) {
    let mut q = RATE_QUEUE.lock().await;
    if q.len() == q.capacity() { q.pop_front(); }
    q.push_back(rec);
}
