use uuid::Uuid;

// These structures and enums are shared between master and slave.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Request {
    Helo,
    RegisterUnit,
    ProtocolError,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Reply {
    Helo(String),
    UnitRegistered(Uuid),
}

