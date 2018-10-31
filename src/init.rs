/* This file is part of the Aeterno init system. */

/* Goal:
 *  - Create the socket for communication with aeterno-sys
 *  - Exec aeterno-sys
 */

extern crate nix;
use nix::sys::socket::{AddressFamily, bind, SockAddr, SockFlag, SockType};
use nix::sys::socket::{socket, UnixAddr};

use std::ffi::CString;

const SYS_SOCKET_PATH: &str = "/run/aeterno/sys.sock";

#[cfg(feature = "native")]
const AETERNO_SYS_PATH: &str = "/sbin/aeterno-sys";

#[cfg(feature = "default")]
const AETERNO_SYS_PATH: &str = "./target/debug/aeterno-sys";

fn main() {
    /* Create the socket */
    let sock_fd = socket(AddressFamily::Unix,
                        SockType::Datagram,
                        SockFlag::empty(),
                        None)
                .expect("FATAL: unable to create socket\n");

    /* Bind the socket to the filesystem */
    let unix_addr: UnixAddr = UnixAddr::new(SYS_SOCKET_PATH)
                .expect("FATAL: Unable to create path for the unix socket\n");
    bind(sock_fd, &SockAddr::Unix(unix_addr))
                .expect("FATAL: Failed to bind socket to address\n");

    /* Fix up fd 4 */
    nix::unistd::dup2(sock_fd, 4)
                .expect("FATAL: Failed to dup2(2) the socket fd\n");

    /* Now that the socket has been created, start spawning aeterno-sys */
    let aeterno_sys_path = CString::new(AETERNO_SYS_PATH).ok().unwrap();
    nix::unistd::execvp(&aeterno_sys_path, &[])
                .expect("FATAL: Failed to start aeterno-sys, panic inbound.\n");
}
