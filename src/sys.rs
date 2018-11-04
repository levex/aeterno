/* This file is part of the Aeterno init system. */

/* Goal:
 *   - Commands must be simple
 *   - Use helper for everything instead of risking pid 1 to crash
 */

extern crate nix;
use std::os::unix::io::RawFd;
use nix::sys::socket::{accept, listen, MsgFlags, recv};
use nix::unistd::{close};

#[macro_use]
extern crate log;
extern crate env_logger;

// const SYS_SOCKET_PATH: &str = "/run/aeterno/sys.sock";
const SYS_SOCKET_FD: RawFd = 4;
const SYS_SOCKET_BACKLOG: usize = 5;

use std::thread;

fn handle_connection(conn_fd: RawFd) {
    debug!("Handling connection for FD {}", conn_fd);
    /* Read in command from the connection */
    let buf: &mut [u8] = &mut [0; 256];
    let size = recv(conn_fd, buf, MsgFlags::empty());
    if size.is_ok() {
        debug!("Received {} bytes\n", size.unwrap());
    } else {
        debug!("Failed to receive from socket\n");
    }

    let _ = close(conn_fd);
}

fn socket_listener() {
    listen(SYS_SOCKET_FD, SYS_SOCKET_BACKLOG)
        .expect("FATAL: cannot listen on the Aeterno socket.");

    loop {
        if let Ok(conn_fd) = accept(SYS_SOCKET_FD) {
            debug!("Accepted a connection with FD {}", conn_fd);
            let r = thread::spawn(move || {
                handle_connection(conn_fd);
            });
            debug!("FD {}: r = {:?}", conn_fd, r);
        }
    }
}

fn main() {
    /* Initialize logging */
    env_logger::init();

    /* TODO: spawn aeterno-master */

    /* Start the socket listener */
    thread::spawn(socket_listener);

    /* The main thread should forever yield */
    loop {}
}
