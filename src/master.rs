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
    bind(master_fd , &SockAddr::Unix(master_unix_addr))
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
    } else {
        error!("Invalid response from aeterno-sys");
    }
}
