/* This file is part of the Aeterno init system. */

/* Goal:
 *   - Commands must be simple
 *   - Use helper for everything instead of risking pid 1 to crash
 */

use std::os::unix::io::RawFd;
use std::str::from_utf8;

extern crate nix;
use nix::sys::socket::{accept, listen, MsgFlags, recv};
use nix::unistd::{close, write};

#[macro_use]
extern crate log;
extern crate env_logger;

// const SYS_SOCKET_PATH: &str = "/run/aeterno/sys.sock";
const SYS_SOCKET_FD: RawFd = 4;
const SYS_SOCKET_BACKLOG: usize = 5;

use std::thread;

#[derive(Debug, PartialEq, Eq)]
enum Query {
    Helo,
    ProtocolError,
}

impl From<&str> for Query {
    fn from(s: &str) -> Query {
        match s {
            "HELO\n" => Query::Helo,
            _ => Query::ProtocolError,
        }
    }
}

fn reply_query(conn_fd: RawFd, q: Query) {
    match q {
        Query::Helo => {
            info!("Received HELO from fd {:?}", conn_fd);
        },
        Query::ProtocolError => {
            info!("Protocol error with fd {:?}", conn_fd);
        },
    }
}

fn handle_connection(conn_fd: RawFd) {
    debug!("Handling connection for FD {}", conn_fd);
    /* Read in command from the connection */
    let buf: &mut [u8] = &mut [0; 256];
    let size = recv(conn_fd, buf, MsgFlags::empty());
    if size.is_ok() {
        debug!("Received {} bytes", size.unwrap());
        let query = from_utf8(&buf)
            .map(|str| { str.trim_matches(char::from(0)) })
            .map(From::from)
            .unwrap_or_else(|err| {
                debug!("{:?}", err);
                Query::ProtocolError
            });

        reply_query(conn_fd, query);
    } else {
        debug!("Failed to receive from socket");
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
            /* FIXME: FDs keep piling up here */
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
