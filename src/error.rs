use std::fmt;

#[derive(Debug, Clone)]
pub enum ApiError {
    Connection(String),
    Response(String),
    Timeout(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Connection(msg) => write!(f, "Connection error: {}", msg),
            ApiError::Response(msg) => write!(f, "Response error: {}", msg),
            ApiError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

pub const DEFAULT_TIMEOUT: u64 = 25;
pub const QUICK_TIMEOUT: u64 = 12;
pub const CONNECTION_TIMEOUT: u64 = 8;
pub const DEEPSEEK_TIMEOUT: u64 = 35;
pub const DEFAULT_RETRIES: u32 = 2;
pub const QUICK_RETRIES: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error_display() {
        let err = ApiError::Connection("Network down".to_string());
        assert_eq!(err.to_string(), "Connection error: Network down");
    }

    #[test]
    fn test_response_error_display() {
        let err = ApiError::Response("Invalid JSON".to_string());
        assert_eq!(err.to_string(), "Response error: Invalid JSON");
    }

    #[test]
    fn test_timeout_error_display() {
        let err = ApiError::Timeout("Request exceeded 25s".to_string());
        assert_eq!(err.to_string(), "Timeout error: Request exceeded 25s");
    }

    #[test]
    fn test_timeout_constants() {
        assert_eq!(DEFAULT_TIMEOUT, 25);
        assert_eq!(QUICK_TIMEOUT, 12);
        assert_eq!(CONNECTION_TIMEOUT, 8);
        assert_eq!(DEEPSEEK_TIMEOUT, 35);
        assert_eq!(DEFAULT_RETRIES, 2);
        assert_eq!(QUICK_RETRIES, 1);
    }
}
