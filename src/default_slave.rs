/* This file is part of the Aeterno init system. */
extern crate bincode;
use bincode::{deserialize, serialize};

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate nix;
use nix::Result;
use nix::sys::socket::{AddressFamily, connect, MsgFlags, SockAddr, SockFlag};
use nix::sys::socket::{SockType, socket, UnixAddr, recv};
use nix::unistd::{close, write};
use std::os::unix::io::RawFd;
use std::path::PathBuf;

#[macro_use]
extern crate serde_derive;

extern crate uuid;
use uuid::Uuid;

const MASTER_SOCKET_PATH: &str = "/run/aeterno/master.sock";

#[cfg(feature = "native")]
const SLAVE_SERVICES: &str = "/usr/local/aeterno/services/";

#[cfg(feature = "default")]
const SLAVE_SERVICES: &str = "./samples/services/";

mod master_slave_shared;
pub use master_slave_shared::*;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct Unit {
    pub name: String,
    pub exec_start: String,

    #[serde(skip)]
    pub uuid: Uuid,
}

fn send_request(fd: RawFd, req: Request) -> Result<usize> {
    serialize(&req)
        .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))
        .and_then(|r| write(fd, r.as_slice()))
}

fn get_reply(fd: RawFd) -> Result<Reply> {
    let buf: &mut [u8] = &mut [0; 256];
    recv(fd, buf, MsgFlags::empty())
        .and_then(|_| {
            deserialize(buf)
                .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))
        })
}

fn send_and_receive(fd: RawFd, req: Request) -> Result<Reply> {
    send_request(fd, req)
        .and_then(|_| { get_reply(fd) })
}

fn register_unit(fd: RawFd, unit: &mut Unit) {
    match send_and_receive(fd, Request::RegisterUnit) {
        Ok(Reply::UnitRegistered(reply)) => { 
            info!("Registered unit: {}", reply);
            unit.uuid = reply;
        },
        a => error!("failed to register a unit! {:?}", a),
    }
}

fn handle_connection(fd: RawFd) -> bool {
    /* Send a helo */
    match send_and_receive(fd, Request::Helo) {
        Ok(Reply::Helo(reply)) => info!("Counterpart version: {}", reply),
        a => error!("failed to get counterpart version! {:?}", a),
    }

    let units = enumerate_units().unwrap_or(Vec::new());
    for mut unit in units {
        register_unit(fd, &mut unit);
    }

    true
}

fn load_unit_at(path: &PathBuf) -> Result<Unit> {
    let contents = std::fs::read_to_string(path);
    if contents.is_err() {
        debug!("load_unit_at: failed to read_to_string: {:?}", contents);
        return Err(nix::Error::from_errno(nix::errno::Errno::EINVAL));
    }
    let contents = contents.unwrap();
    let unit = toml::from_str(&contents);
    if unit.is_ok() {
        let unit = unit.unwrap();
        debug!("loaded unit {:?}", unit);
        return Ok(unit);
    } else {
        debug!("load_unit_at: failed to from_str: {:?}", unit);
        return Err(nix::Error::from_errno(nix::errno::Errno::EINVAL));
    }
}

fn enumerate_units() -> Result<Vec<Unit>> {
    let paths = std::fs::read_dir(SLAVE_SERVICES);
    let mut units: Vec<Unit> = Vec::new();
    if paths.is_ok() {
        for path in paths.unwrap() {
            let path = path.unwrap();
            info!("Loading service {}", path.path().display());
            let unit = load_unit_at(&path.path());
            if unit.is_ok() {
                units.push(unit.unwrap());
            } else {
                warn!("failed to load service {}: {:?}",
                      path.path().display(), unit);
            }
        }

        return Ok(units);
    }

    Err(nix::Error::from_errno(nix::errno::Errno::ENOENT))
}

fn main() {
    env_logger::init();
    info!("default-slave starting...");

    /* Open a socket to the master */
    let master_fd = socket(AddressFamily::Unix,
                        SockType::Stream,
                        SockFlag::empty(),
                        None)
        .expect("FATAL: failed to create master socket counterpair");

    let master_unix_addr: UnixAddr = UnixAddr::new(MASTER_SOCKET_PATH)
                .expect("FATAL: Unable to create path for the unix socket");
    connect(master_fd,  &SockAddr::Unix(master_unix_addr))
        .expect("FATAL: Failed to connect to master socket");

    let r = handle_connection(master_fd);
    if r {
        let _ = close(master_fd);
    } else {
        /* TODO: reconnect? */
    }
}
