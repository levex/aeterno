/* This file is part of the Aeterno init system. */
use bincode::{deserialize, serialize};

use nix::sys::socket::{accept, MsgFlags, recv};
use nix::unistd::{close, write};

use std::os::unix::io::RawFd;
use std::thread;

use uuid::Uuid;

mod master_slave_shared;
use master_slave_shared::{Reply, Request};

use ::SysReply::*;

fn handle_helo(conn_fd: RawFd) -> bool {
    let helo = Reply::Helo("aeterno-master 0.0.1 - November 2018".to_string());
    let encoded: Vec<u8> = serialize(&helo).unwrap();

    let _ = write(conn_fd, encoded.as_slice());
    false
}

fn handle_register_unit(conn_fd: RawFd) -> bool {
    let uuid = Uuid::new_v4();
    let reply = Reply::UnitRegistered(uuid);
    let encoded: Vec<u8> = serialize(&reply).unwrap();

    use ::register_unit;
    register_unit(conn_fd, uuid);

    let _ = write(conn_fd, encoded.as_slice());
    false
}

fn handle_unit_start_executable(sys_fd: RawFd, conn_fd: RawFd, uuid: Uuid,
                                execstr: String) -> bool {
    debug!("Handling Start request for fd {} uuid {} execstr \"{}\"",
           conn_fd, uuid, execstr);

    let _ = write(sys_fd, format!("START {}", execstr).as_bytes());

    /* We should now receive either `ERR XX` or `OK XX`,
     * where in the case of `ERR`, `XX` is the errno from the execve(2) call.
     *
     * In the case of `OK`, the `XX` is the PID of the process created.
     */
    use ::sys_reply;
    let res = sys_reply(sys_fd);

    match res {
        Ok(Okay(pid)) => info!("spawned process with pid {}", pid),
        Ok(Error(e)) => info!("failed to spawn process {:?}", e),
        Err(e) => info!("parse error of sys reply: {:?}", e),
    }

    false
}

fn handle_request(sys_fd: RawFd, conn_fd: RawFd, req: Request) -> bool {
    match req {
        Request::Helo => handle_helo(conn_fd),
        Request::RegisterUnit => handle_register_unit(conn_fd),
        Request::UnitStartExecutable(uuid, execstr)
            => handle_unit_start_executable(sys_fd, conn_fd, uuid, execstr),
        _ => true,
    }
}

fn handle_connection(sys_fd: RawFd, conn_fd: RawFd) {
    let buf: &mut [u8] = &mut [0; 256];

    loop {
        let size = recv(conn_fd, buf, MsgFlags::empty());
        if size.is_ok() && size.unwrap() > 0 {
            let msg: Request = deserialize(buf)
                .unwrap_or(Request::ProtocolError);
            let should_close = handle_request(sys_fd, conn_fd, msg);
            if should_close {
                let _ = close(conn_fd);
                break;
            }
        }
    }
}

/// Start listening on a socket, expecting connections from slaves
///
/// Assumes that listening has already been setup by the caller.
#[allow(dead_code)]
pub fn start_listening(sys_fd: RawFd, fd: RawFd) {
    loop {
        if let Ok(conn_fd) = accept(fd) {
            debug!("Accepted a connection with FD {}", conn_fd);
            thread::spawn(move || {
                handle_connection(sys_fd, conn_fd);
            });
        }
    }
}
