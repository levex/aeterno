/* This file is part of the Aeterno init system */

/* Goal:
 * - Keep track of different Units.
 * - Connect to the -sys and get a master connection, using it to receive wait
 *   events
 */

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate nix;
use nix::Result;
use nix::sys::socket::{AddressFamily, connect, bind, listen, SockAddr, SockFlag};
use nix::sys::socket::{SockType, socket, UnixAddr};
use nix::unistd::{read, write};

use std::os::unix::io::RawFd;

const MASTER_SOCKET_PATH: &str = "/run/aeterno/master.sock";
const SYS_SOCKET_PATH: &str = "/run/aeterno/sys.sock";

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
struct SysVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
enum SysReply {
    Okay(u64),
    Error(u64),
}

/// Read in the version from the aeterno system by executing a HELO command
fn sys_version(sys_fd: RawFd) -> Result<SysVersion> {
    let buf = &mut [0u8; 128];

    /* retrieve version information */
    write(sys_fd, "HELO\n".as_bytes())?;

    /* read back the information */
    let len = read(sys_fd, buf)?;

    /* Verify protocol */
    if len > 8 {
        let aeterno_str = std::str::from_utf8(buf)
            .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))?;

        let explosion = aeterno_str.split(|c| c == ' ' || c == '.')
            .collect::<Vec<_>>();

        /* verify that it is Aeterno we are talking to */
        if explosion[0] != "Aeterno" {
            return Err(nix::Error::Sys(nix::errno::Errno::EBADF));
        }

        let major = explosion[1].parse()
            .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))?;
        let minor = explosion[2].parse()
            .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))?;
        let patch = explosion[3].parse()
            .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))?;

        Ok(SysVersion {
            major,
            minor,
            patch,
        })
    } else { 
        Err(nix::Error::Sys(nix::errno::Errno::EBADF))
    }
}

/// Collects a reply from the socket and parses it
fn sys_reply(sys_fd: RawFd) -> Result<SysReply> {
    let buf = &mut [0u8; 128];

    read(sys_fd, buf)?;

    /* convert into a string */
    let converted = std::str::from_utf8(buf)
        .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))?;

    /* explode the string */
    let explosion = converted.split(|c| c == ' ' || c == '.' || c == '\n')
        .collect::<Vec<_>>();

    /* sanity */
    if explosion.len() < 2 {
        return Err(nix::Error::Sys(nix::errno::Errno::EINVAL));
    }

    /* extract the information */
    let control = explosion[0];
    let value = explosion[1];

    /* construct the final value */
    match control {
        "OK" => {
            let v = value.parse::<u64>()
                .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))?;
            Ok(SysReply::Okay(v))
        },
        "ERR" => {
            let v = value.parse::<u64>()
                .or(Err(nix::Error::Sys(nix::errno::Errno::EINVAL)))?;
            Ok(SysReply::Error(v))
        },
        _ => {
            Err(nix::Error::Sys(nix::errno::Errno::EINVAL))
        }
    }
}

/// Asks the sys instance to check whether this connection is a mastering connection
fn check_mastering(sys_fd: RawFd) -> bool {
    let r = write(sys_fd, "MASTER\n".as_bytes());
    if r.is_err() {
        return false;
    }

    let reply = sys_reply(sys_fd);
    if reply.is_err() {
        return false;
    }

    /* This unwrap is safe, because of the is_err() check before-hand. */
    let reply = reply.unwrap();

    match reply {
        SysReply::Okay(_) => true,
        _ => false,
    }
}

fn main() {
    env_logger::init();
    info!("aeterno-master start up");

    /* Create master.sock */
    let master_fd = socket(AddressFamily::Unix,
                        SockType::Stream,
                        SockFlag::empty(),
                        None)
                .expect("FATAL: unable to create socket");

    /* Bind the socket to the filesystem */
    let master_unix_addr: UnixAddr = UnixAddr::new(MASTER_SOCKET_PATH)
                .expect("FATAL: Unable to create path for the unix socket");
    bind(master_fd, &SockAddr::Unix(master_unix_addr))
                .expect("FATAL: Failed to bind socket to address");

    /* Start listening */
    listen(master_fd , 5)
        .expect("FATAL: cannot listen on the Aeterno socket.");

    /* Open the sys socket */
    let sys_fd = socket(AddressFamily::Unix,
                        SockType::Stream,
                        SockFlag::empty(),
                        None)
        .expect("FATAL: failed to create sys socket counterpair");

    let sys_unix_addr: UnixAddr = UnixAddr::new(SYS_SOCKET_PATH)
                .expect("FATAL: Unable to create path for the unix socket");
    connect(sys_fd,  &SockAddr::Unix(sys_unix_addr))
        .expect("FATAL: Failed to connect to sys socket");

    if let Ok(ver) = sys_version(sys_fd) {
        info!("Aeterno Sys Version {:?}", ver);

        let mastering = check_mastering(sys_fd);
        if !mastering {
            error!("This master instance is not mastering the aeterno sys");
        } else {
            info!("Acquired sys mastering for this instance");
        }
    } else {
        error!("Invalid response from aeterno-sys");
    }
}
