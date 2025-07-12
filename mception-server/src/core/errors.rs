use std::fmt;

/// Common result type used throughout the application
pub type MceptionResult<T> = Result<T, MceptionError>;

/// Top-level error type for the MCeption system
#[derive(Debug)]
pub enum MceptionError {
    Storage(StorageError),
    Configuration(ConfigurationError),
    Network(NetworkError),
    Validation(ValidationError),
}

/// Errors related to data storage operations
#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    NotFound(String),
    AlreadyExists(String),
    Corruption(String),
}

/// Errors related to configuration management
#[derive(Debug)]
pub enum ConfigurationError {
    InvalidConfiguration(String),
    MissingRequiredField(String),
    ConflictingSettings(String),
}

/// Errors related to network operations
#[derive(Debug)]
pub enum NetworkError {
    ConnectionFailed(String),
    Timeout(String),
    InvalidUrl(String),
}

/// Errors related to data validation
#[derive(Debug)]
pub enum ValidationError {
    InvalidFormat(String),
    ValueOutOfRange(String),
    RequiredFieldMissing(String),
}

// Implement From traits for common error conversions
impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization(err)
    }
}

impl From<StorageError> for MceptionError {
    fn from(err: StorageError) -> Self {
        MceptionError::Storage(err)
    }
}

impl From<ConfigurationError> for MceptionError {
    fn from(err: ConfigurationError) -> Self {
        MceptionError::Configuration(err)
    }
}

impl From<NetworkError> for MceptionError {
    fn from(err: NetworkError) -> Self {
        MceptionError::Network(err)
    }
}

impl From<ValidationError> for MceptionError {
    fn from(err: ValidationError) -> Self {
        MceptionError::Validation(err)
    }
}

// Display implementations
impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(err) => write!(f, "IO error: {}", err),
            StorageError::Serialization(err) => write!(f, "Serialization error: {}", err),
            StorageError::NotFound(resource) => write!(f, "Resource not found: {}", resource),
            StorageError::AlreadyExists(resource) => write!(f, "Resource already exists: {}", resource),
            StorageError::Corruption(details) => write!(f, "Data corruption detected: {}", details),
        }
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::InvalidConfiguration(details) => write!(f, "Invalid configuration: {}", details),
            ConfigurationError::MissingRequiredField(field) => write!(f, "Missing required field: {}", field),
            ConfigurationError::ConflictingSettings(details) => write!(f, "Conflicting settings: {}", details),
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ConnectionFailed(details) => write!(f, "Connection failed: {}", details),
            NetworkError::Timeout(details) => write!(f, "Operation timed out: {}", details),
            NetworkError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidFormat(details) => write!(f, "Invalid format: {}", details),
            ValidationError::ValueOutOfRange(details) => write!(f, "Value out of range: {}", details),
            ValidationError::RequiredFieldMissing(field) => write!(f, "Required field missing: {}", field),
        }
    }
}

impl fmt::Display for MceptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MceptionError::Storage(err) => write!(f, "Storage error: {}", err),
            MceptionError::Configuration(err) => write!(f, "Configuration error: {}", err),
            MceptionError::Network(err) => write!(f, "Network error: {}", err),
            MceptionError::Validation(err) => write!(f, "Validation error: {}", err),
        }
    }
}

// Implement std::error::Error
impl std::error::Error for StorageError {}
impl std::error::Error for ConfigurationError {}
impl std::error::Error for NetworkError {}
impl std::error::Error for ValidationError {}
impl std::error::Error for MceptionError {}
