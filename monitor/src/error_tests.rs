#[cfg(test)]
mod error_tests {
    use crate::error::DashboardError;

    #[test]
    fn test_error_display_write_failed() {
        let err = DashboardError::WriteFailed("connection refused".to_string());
        assert_eq!(err.to_string(), "InfluxDB write failed: connection refused");
    }

    #[test]
    fn test_error_display_query_failed() {
        let err = DashboardError::QueryFailed("timeout".to_string());
        assert_eq!(err.to_string(), "InfluxDB query failed: timeout");
    }

    #[test]
    fn test_error_display_config() {
        let err = DashboardError::Config("missing field".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing field");
    }

    #[test]
    fn test_error_display_storage() {
        let err = DashboardError::Storage("lock poisoned".to_string());
        assert_eq!(err.to_string(), "Storage error: lock poisoned");
    }

    #[test]
    fn test_error_debug() {
        let err = DashboardError::WriteFailed("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("WriteFailed"));
    }
}
