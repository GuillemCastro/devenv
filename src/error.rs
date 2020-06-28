use std::error::Error as StdError;
use std::fmt;
use libmount::Error;

#[derive(Debug)]
pub struct MountError {
    source: Error
}

impl StdError for MountError {
    
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }

}

impl fmt::Display for MountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error")
    }
}