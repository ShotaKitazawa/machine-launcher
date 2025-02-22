use ping::ping;
use ping::Error as PingError;
use wakey::WakeyError;
use wakey::WolPacket;

use crate::{Error, PowerManagerTrait, PowerStatus};

impl From<WakeyError> for Error {
    fn from(e: WakeyError) -> Self {
        Error::InternalServerError(format!("WakeyError: {}", e).into())
    }
}
impl From<PingError> for Error {
    fn from(e: PingError) -> Self {
        Error::InternalServerError(format!("PingError: {}", e).into())
    }
}

#[derive(Debug, Clone)]
pub struct WakeOnLanDriver {
    name: String,
    client: WolPacket,
    ip_addr: std::net::IpAddr,
}
impl WakeOnLanDriver {
    pub fn new<T: AsRef<str>>(
        name: String,
        mac_addr: T,
        ip_addr_unparsed: T,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ip_addr = ip_addr_unparsed.as_ref().to_string().parse()?;
        let client = WolPacket::from_string(mac_addr, ':')?;
        Ok(WakeOnLanDriver {
            name,
            client,
            ip_addr,
        })
    }
}
impl PowerManagerTrait for WakeOnLanDriver {
    fn start(&self) -> Result<(), Error> {
        let res = self.client.send_magic();
        match res {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from(e)),
        }
    }
    fn status(&self) -> Result<PowerStatus, Error> {
        // MEMO: ping will work only on Linux because this library uses rawsocket
        // TODO: update here to work well
        let res = ping(
            self.ip_addr,
            Some(std::time::Duration::from_secs(1)),
            Some(64),
            None,
            Some(1),
            None,
        );
        match res {
            Ok(()) => Ok(PowerStatus {
                name: self.name.clone(),
                hostname: self.ip_addr.to_string(),
                running: true,
                reason: None,
            }),
            Err(e) => match e {
                PingError::InternalError => Err(Error::from(e)),
                _ => Ok(PowerStatus {
                    name: self.name.clone(),
                    hostname: self.ip_addr.to_string(),
                    running: false,
                    reason: Some(e.to_string()),
                }),
            },
        }
    }
    fn stop(&self) -> Result<(), Error> {
        Err(Error::NotImplemented())
    }
}
