use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::{Arc, Mutex},
};

use chrono::Local;
use rust_ipmi::{IPMIClient, IPMIClientError};

use crate::{Error, PowerManagerTrait, PowerStatus};

const CLIENT_RENEW_PERIOD: i64 = 60;

impl From<IPMIClientError> for Error {
    fn from(e: IPMIClientError) -> Self {
        Error::InternalServerError(format!("IPMIClientError: {}", e).into())
    }
}

#[derive(Debug, Clone)]
pub struct IpmiDriver {
    name: String,
    server_addr: SocketAddr,
    client: Arc<Mutex<IpmiClient>>,
}
impl IpmiDriver {
    pub fn new(
        name: String,
        server_addr_unparsed: String,
        username: String,
        password: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let server_addr = server_addr_unparsed
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| Error::InternalServerError("server_addr is invalid".into()))?;
        let client = Arc::new(Mutex::new(IpmiClient::new(
            server_addr,
            username.clone(),
            password.clone(),
        )?));
        Ok(IpmiDriver {
            name,
            server_addr,
            client,
        })
    }
}

impl PowerManagerTrait for IpmiDriver {
    fn start(&self) -> Result<(), Error> {
        let mut client = self.client.lock().unwrap();
        client.renew()?;

        let netfn = 0x00; // Chassis NetFn
        let cmd = 0x02; // Chassis Control
        let data = Some(vec![0x01]); // Power On
        match client.c.send_raw_request(netfn, cmd, data) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::from(e)),
        }
    }
    fn status(&self) -> Result<PowerStatus, Error> {
        let mut client = self.client.lock().unwrap();
        client.renew()?;

        let netfn = 0x00; // Chassis NetFn
        let cmd = 0x01; // Chassis Status
        match client.c.send_raw_request(netfn, cmd, None) {
            Ok(resp) => {
                let data = resp.data.ok_or_else(|| {
                    Error::InternalServerError("Invalid response received: data is empty".into())
                })?;
                if data.is_empty() {
                    return Err(Error::InternalServerError(
                        "Invalid response received: data is empty".into(),
                    ));
                }
                match data[0] {
                    0x01 => Ok(PowerStatus {
                        name: self.name.clone(),
                        hostname: self.server_addr.to_string(),
                        running: true,
                        reason: None,
                    }),
                    _ => Ok(PowerStatus {
                        name: self.name.clone(),
                        hostname: self.server_addr.to_string(),
                        running: false,
                        reason: None,
                    }),
                }
            }
            Err(e) => Err(Error::from(e)),
        }
    }
    fn stop(&self) -> Result<(), Error> {
        let mut client = self.client.lock().unwrap();
        client.renew()?;

        let netfn = 0x00; // Chassis NetFn
        let cmd = 0x02; // Chassis Control
        let data = Some(vec![0x00]); // Power Off
        match client.c.send_raw_request(netfn, cmd, data) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::from(e)),
        }
    }
}

#[derive(Debug)]
pub struct IpmiClient {
    c: IPMIClient,
    server_addr: SocketAddr,
    username: String,
    password: String,
    timestamp: i64,
}
impl IpmiClient {
    fn new(server_addr: SocketAddr, username: String, password: String) -> Result<Self, Error> {
        let mut c = IPMIClient::new(server_addr)?;
        c.establish_connection(&username, &password)?;
        Ok(IpmiClient {
            c,
            server_addr,
            username,
            password,
            timestamp: Local::now().timestamp(),
        })
    }
    fn renew(&mut self) -> Result<(), Error> {
        let now = Local::now().timestamp();
        if now - self.timestamp > CLIENT_RENEW_PERIOD {
            let new_client = IpmiClient::new(
                self.server_addr,
                self.username.clone(),
                self.password.clone(),
            )?;
            self.timestamp = now;
            self.c = new_client.c;
        }
        Ok(())
    }
}
