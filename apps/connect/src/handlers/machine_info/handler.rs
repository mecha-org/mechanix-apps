use std::io::Error;

use anyhow::{bail, Ok, Result};

use crate::server::settings_client::SettingsClient;

pub struct MachineHandler {
    machine_id: Option<String>,
    
}

impl MachineHandler {

    pub fn new() -> Result<Self> {
       Ok(Self {
        machine_id: None,
     })
    }

    pub fn run() {

    }


}

