#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    /// Request Pong with same data
    Ping(u32),
    /// Ping reply
    Pong(u32),
    /// “useful” data transfer
    Data(String),

    /// Host -> Puncher registration
    Register,
    /// Puncher -> Host id assignment
    RegisterReply(u64),
    /// Client -> Puncher id request
    Query(u64),
    /// Puncher -> Host/Client connection request (socket address)
    ConnectTo(String), // TODO: More efficient representation
    /// Puncher -> Client for invalid id
    NotFound(u64),
}
