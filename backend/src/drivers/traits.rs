use crate::Error;

pub trait PowerManagerTrait: Send + Sync {
    fn start(&self) -> Result<(), Error>;
    fn status(&self) -> Result<PowerStatus, Error>;
    fn stop(&self) -> Result<(), Error>;
}

pub struct PowerStatus {
    pub name: String,
    pub hostname: String,
    pub running: bool,
    pub reason: Option<String>,
}
