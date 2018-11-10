/* This file is part of the Aeterno init system. */

/* Goal:
 *   - Commands must be simple
 *   - Use helper for everything instead of risking pid 1 to crash
 */

use std::os::unix::io::RawFd;
use std::str::from_utf8;
use std::path::PathBuf;

extern crate nix;
use nix::sys::socket::{accept, listen, MsgFlags, recv};
use nix::sys::signal::kill;
use nix::unistd::{close, Pid, write};

#[macro_use]
extern crate log;
extern crate env_logger;

// const SYS_SOCKET_PATH: &str = "/run/aeterno/sys.sock";
const SYS_SOCKET_FD: RawFd = 4;
const SYS_SOCKET_BACKLOG: usize = 5;
const AETERNO_VERSION: &str = "Aeterno v0.0.1 - November 2018\n";

use std::thread;

#[derive(Debug, PartialEq, Eq)]
enum RawQuery {
    Helo,
    Start(String),
    Stop(String),
    ProtocolError,
}

#[derive(Debug, PartialEq, Eq)]
enum Query {
    Helo,
    Start(PathBuf),
    Stop(Pid),
    ProtocolError,
}

macro_rules! conn_ok {
    ($fd:expr) => {
        let _ = write($fd, "OK\n".as_bytes());
    }
}

impl From<&str> for RawQuery {
    fn from(s: &str) -> RawQuery {
        match s.split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["HELO"] => RawQuery::Helo,
            ["START", x] => RawQuery::Start(x.to_string()),
            ["STOP", x] => RawQuery::Stop(x.to_string()),
            _ => RawQuery::ProtocolError,
        }
    }
}

fn reply_query(conn_fd: RawFd, q: Query) {
    match q {
        Query::Helo => {
            info!("Received HELO from fd {:?}", conn_fd);
            /*
             * Write version string back to the connection,
             * don't care if it fails
             */
            let _ = write(conn_fd, AETERNO_VERSION.as_bytes());
        },
        Query::Start(path) => {
            info!("Received START {:?} command from fd {:?}",
                  path, conn_fd);
        },
        Query::Stop(pid) => {
            info!("Received STOP {:?} command from fd {:?}",
                  pid, conn_fd);
        },
        Query::ProtocolError => {
            info!("Protocol error with fd {:?}", conn_fd);
        },
    }
}

/* TODO: convert this to a Result type */
fn validate_raw_query(rq: RawQuery) -> Option<Query> {
    match rq {
        RawQuery::Helo => Some(Query::Helo),
        RawQuery::ProtocolError => Some(Query::ProtocolError),
        RawQuery::Start(path_str) => {
            let mut p = PathBuf::new();
            p.push(path_str);

            /* Verify that the path is valid */
            if p.exists() {
                Some(Query::Start(p))
            } else {
                None
            }
        },
        RawQuery::Stop(pid_str) => {
            let pid_i = pid_str
                .parse::<i32>()
                .ok();
            if let Some(p) = pid_i {
                let pid = Pid::from_raw(p);

                /* Verify that the PID is valid */
                if kill(pid, None).is_ok() {
                    Some(Query::Stop(pid))
                } else {
                    None
                }
            } else {
                None
            }
        },
    }
}

fn handle_connection(conn_fd: RawFd) {
    debug!("Handling connection for FD {}", conn_fd);
    /* Read in command from the connection */
    let buf: &mut [u8] = &mut [0; 256];
    let size = recv(conn_fd, buf, MsgFlags::empty());
    if size.is_ok() {
        /* The .unwrap() here is OK, since we check is_ok() before */
        debug!("Received {} bytes", size.unwrap());
        let query = from_utf8(&buf)
            .map(|str| { str.trim_matches(char::from(0)) })
            .map(From::from)
            .unwrap_or_else(|err| {
                debug!("{:?}", err);
                RawQuery::ProtocolError
            });

        if let Some(q) = validate_raw_query(query) {
            reply_query(conn_fd, q);
        } else {
            /* TODO: forward the unix error */
            let _ = write(conn_fd, "ERR -1\n".as_bytes());
        }
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
