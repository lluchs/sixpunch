use super::super::{Message, Puncher as PuncherTrait};
use std::net::{UdpSocket, SocketAddr};
use std::collections::HashMap;
use rand::{self, Rng};
use bincode::{self, serialize, deserialize};

pub struct Puncher {
    addrs: HashMap<u64, SocketAddr>,
    socket: UdpSocket,
    limit: bincode::SizeLimit,
}

impl Puncher {
    pub fn new(addr: &str) -> Puncher {
        Puncher {
            addrs: HashMap::new(),
            socket: UdpSocket::bind(addr).unwrap(),
            limit: bincode::SizeLimit::Bounded(500),
        }
    }

    fn handle_msg(&mut self, msg: Message, src_addr: &SocketAddr) -> Option<Message> {
        match msg {
            Message::Ping(x) => Some(Message::Pong(x)),
            Message::Register => Some(Message::RegisterReply(self.register(src_addr))),
            Message::Query(id) =>
                if let Some(addr) = self.query(id, src_addr) {
                    Some(Message::ConnectTo(addr))
                } else {
                    Some(Message::NotFound(id))
                },
            _ => None
        }
    }

    fn register(&mut self, src_addr: &SocketAddr) -> u64 {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<u64>();
        self.addrs.insert(id, src_addr.clone());
        id
    }

    fn query(&mut self, id: u64, src_addr: &SocketAddr) -> Option<String> {
        if let Some(addr) = self.addrs.get(&id) {
            // Ask the host to connect to our querying client.
            self.socket.send_to(&serialize(&Message::ConnectTo(format!("{}", src_addr)), self.limit).unwrap(), addr)
                       .expect("send to host failed");
            // Reply address of host to client.
            Some(format!("{}", addr))
        } else {
            None
        }
    }
}

impl PuncherTrait for Puncher {
    fn run(&mut self) {
        let mut buf = [0; 500];
        loop {
            let (number_of_bytes, src_addr)
                = self.socket.recv_from(&mut buf)
                      .expect("Didn't receive data");
            let msg: Message = deserialize(&buf).expect("Deserialization failure");

            println!("recv({:?}): {:?}", src_addr, msg);
            if let Some(reply) = self.handle_msg(msg, &src_addr) {
                println!("send({:?}): {:?}", src_addr, reply);
                self.socket.send_to(&serialize(&reply, self.limit).unwrap(), src_addr)
                           .expect("reply failed");
            }
        }
    }
}
