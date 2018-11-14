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
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::sys::socket::{AddressFamily, bind, listen, SockAddr, SockFlag, SockType};
use nix::sys::socket::{socket, UnixAddr};

const MASTER_SOCKET_PATH: &str = "/run/aeterno/master.sock";
const SYS_SOCKET_PATH: &str = "/run/aeterno/sys.sock";

fn main() {
    env_logger::init();
    info!("aeterno-master start up");

    /* Create master.sock */
    let master_fd  = socket(AddressFamily::Unix,
                        SockType::Stream,
                        SockFlag::empty(),
                        None)
                .expect("FATAL: unable to create socket\n");

    /* Bind the socket to the filesystem */
    let unix_addr: UnixAddr = UnixAddr::new(MASTER_SOCKET_PATH)
                .expect("FATAL: Unable to create path for the unix socket\n");
    bind(master_fd , &SockAddr::Unix(unix_addr))
                .expect("FATAL: Failed to bind socket to address\n");

    /* Start listening */
    listen(master_fd , 5)
        .expect("FATAL: cannot listen on the Aeterno socket.");

    /* Open the sys socket */
    let sys_fd = open(SYS_SOCKET_PATH, OFlag::O_RDWR, Mode::empty())
        .expect("FATAL: sys socket not found\n");

    /* TODO:
     *   - Retrieve version information and verify compatibility
     *   - Check that sys_fd is a master connection
     */
}
