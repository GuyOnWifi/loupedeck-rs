//! Error types and Result alias for the Loupedeck driver.

use std::fmt;

/// Custom error type for Loupedeck operations.
#[derive(Debug)]
pub enum LoupedeckError {
    /// Errors from the underlying serial port communication.
    Serial(serialport::Error),
    /// Standard I/O errors (timeouts, read/write failures).
    Io(std::io::Error),
}

impl fmt::Display for LoupedeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoupedeckError::Serial(e) => write!(f, "Serial error: {}", e),
            LoupedeckError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for LoupedeckError {}

impl From<serialport::Error> for LoupedeckError {
    fn from(e: serialport::Error) -> Self {
        LoupedeckError::Serial(e)
    }
}

impl From<std::io::Error> for LoupedeckError {
    fn from(e: std::io::Error) -> Self {
        LoupedeckError::Io(e)
    }
}

/// A convenient type alias for Results within this crate.
pub type Result<T> = std::result::Result<T, LoupedeckError>;
