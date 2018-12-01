/* This file is part of the Aeterno init system. */

// These structures and enums are shared between master and slave.
//
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Request {
    Helo,
    ProtocolError,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Reply {
    Helo(String),
}
