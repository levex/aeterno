/* This file is part of the Aeterno init system. */

/* Goal:
 *   - Commands must be simple
 *   - Use helper for everything instead of risking pid 1 to crash
 */

use std::os::unix::io::RawFd;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;

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
    Start(PathBuf, Vec<String>),
    Stop(Pid),
    ProtocolError,
}

macro_rules! conn_ok_with_arg {
    ($fd:expr, $arg:expr) => {
        {
            let _ = write($fd, format!("OK {:?}\n", $arg).as_bytes());
        }
    }
}

macro_rules! conn_ok {
    ($fd:expr) => {
        conn_ok_with_arg!($fd, 0);
    }
}

macro_rules! conn_err {
    ($fd:expr, $arg:expr) => {
        {
            let _ = write($fd, format!("ERR {:?}\n", $arg).as_bytes());
        }
    }
}

fn parse_raw_query(s: &str) -> Option<(&str, String)> {
    let v: Vec<&str> = s.split_whitespace().collect();
    let cmd = v.get(0)?;
    let rest = v.get(1..)
            .map(|x| x.join(" "))
            .unwrap_or("".to_string());

    Some((cmd, rest))
}

impl From<&str> for RawQuery {
    fn from(s: &str) -> RawQuery {
        match parse_raw_query(s) {
            Some(("HELO", x)) => {
                if x == "".to_string() {
                    RawQuery::Helo
                } else {
                    RawQuery::ProtocolError
                }
            },
            Some(("START", x)) => RawQuery::Start(x),
            Some(("STOP", x)) => {
                if x.split_whitespace().count() == 1 {
                    RawQuery::Stop(x)
                } else {
                    RawQuery::ProtocolError
                }
            },
            _ => RawQuery::ProtocolError,
        }
    }
}

fn start_process(conn_fd: RawFd, path: PathBuf, args: Vec<String>) {
    debug!("Starting process {:?} with arguments {:?}", path, args);

    match Command::new(path).args(args).spawn() {
        Ok(child) => conn_ok_with_arg!(conn_fd, child.id()),
        Err(e) => conn_err!(conn_fd, e.raw_os_error().unwrap_or(-1)),
    }
}

fn stop_process(conn_fd: RawFd, pid: Pid) {
    debug!("Stopping process {:?}", pid);

    conn_ok!(conn_fd);
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
        Query::Start(path, args) => {
            info!("Received START {:?} command from fd {:?}",
                  path, conn_fd);

            start_process(conn_fd, path, args);
        },
        Query::Stop(pid) => {
            info!("Received STOP {:?} command from fd {:?}",
                  pid, conn_fd);

            stop_process(conn_fd, pid);
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
            let path_exploded = path_str.split_whitespace()
                .map(str::to_string)
                .collect::<Vec<String>>();
            p.push(path_exploded[0].clone());

            /* Verify that the path is valid */
            if p.exists() {
                Some(Query::Start(p, path_exploded[1..].into()))
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
            conn_err!(conn_fd, -1);
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
