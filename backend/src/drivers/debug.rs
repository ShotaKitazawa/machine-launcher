use std::sync::{Arc, RwLock};

use crate::{Error, PowerManagerTrait, PowerStatus};

#[derive(Debug, Clone)]
pub struct DebugDriver {
    name: String,
    status: Arc<RwLock<bool>>,
}
impl DebugDriver {
    pub fn new(name: String) -> Self {
        DebugDriver {
            name,
            status: Arc::new(RwLock::new(false)),
        }
    }
}
impl PowerManagerTrait for DebugDriver {
    fn start(&self) -> Result<(), Error> {
        println!("{:?}: start", self.name);
        *self.status.write().unwrap() = true;
        Ok(())
    }
    fn status(&self) -> Result<PowerStatus, Error> {
        println!("{:?}: status", self.name);
        Ok(PowerStatus {
            name: self.name.clone(),
            hostname: String::from("dummy1.example.com"),
            running: *self.status.read().unwrap(),
            reason: None,
        })
    }
    fn stop(&self) -> Result<(), Error> {
        println!("{:?}: stop", self.name);
        *self.status.write().unwrap() = false;
        Ok(())
    }
}
