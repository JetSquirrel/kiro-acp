//! Session management types

use serde::{Deserialize, Serialize};

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub cwd: String,
}

impl Session {
    pub fn new(id: String, cwd: String) -> Self {
        Self { id, cwd }
    }
}
