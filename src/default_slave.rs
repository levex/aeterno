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

#[macro_use]
extern crate serde_derive;

const MASTER_SOCKET_PATH: &str = "/run/aeterno/master.sock";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Request {
    Helo,
    ProtocolError,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Reply {
    Helo(String),
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

fn handle_connection(fd: RawFd) -> bool {
    /* Send a helo */
    let reply = send_and_receive(fd, Request::Helo);

    debug!("reply = {:?}", reply);

    true
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
        close(master_fd);
    } else {
        /* TODO: reconnect? */
    }
}
