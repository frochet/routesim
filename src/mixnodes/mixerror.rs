use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::ParseBoolError;


#[derive(Debug)]
pub struct MixError {
    details: String
}

impl MixError {
    pub fn new(msg: &str) -> MixError {
        MixError { details: msg.to_string()}
    }
}

impl fmt::Display for MixError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl From<ParseIntError> for MixError {
    fn from(err: ParseIntError) -> Self {
        MixError::new(&err.to_string())
    }
}
impl From<ParseBoolError> for MixError {
    fn from(err: ParseBoolError) -> Self {
        MixError::new(&err.to_string())
    }
}

impl Error for MixError {
    fn description(&self) -> &str {
        &self.details
    }
}



