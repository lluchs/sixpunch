#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate rand;

pub mod udp;

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

pub trait Puncher {
    fn run(&mut self);
}

pub trait Client {
    fn register_host(&mut self) -> u64;
    fn wait(&mut self);
    fn connect_to_host(&mut self, id: u64) -> bool;
    fn broadcast_data(&mut self, data: &str);
}
