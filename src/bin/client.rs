extern crate bincode;
extern crate rand;
extern crate clap;
extern crate sixpunch;

use sixpunch::Message;
use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use std::io::{self, Write};
use rand::Rng;
use bincode::{serialize, deserialize};
use clap::{Arg, App, SubCommand};

struct Client {
    socket: UdpSocket,
    puncher_addr: SocketAddr,
    peers: Vec<SocketAddr>,
    limit: bincode::SizeLimit,
}

impl Client {
    pub fn new<T, U>(listen_addr: T, puncher_addr: U) -> Client
        where T: ToSocketAddrs, U: ToSocketAddrs {
        Client {
            socket: UdpSocket::bind(listen_addr).expect("couldn't bind socket"),
            puncher_addr: puncher_addr.to_socket_addrs().expect("to_socket_addrs").next().unwrap(),
            peers: Vec::new(),
            limit: bincode::SizeLimit::Bounded(500),
        }
    }

    pub fn register_host(&mut self) -> u64 {
        self.send_puncher_msg(Message::Register)
            .expect("puncher register sendto failed");
        loop {
            let (msg, src_addr) = self.recv_msg();
            match msg {
                Message::RegisterReply(id) => { return id; },
                other => self.handle_msg(other, &src_addr)
            }
        }
    }

    pub fn connect_to_host(&mut self, id: u64) -> bool {
        self.send_puncher_msg(Message::Query(id))
            .expect("puncher query sendto failed");
        let (msg, src_addr) = self.recv_msg();
        match msg {
            Message::NotFound(id) => return false,
            other => self.handle_msg(other, &src_addr)
        }
        true
    }

    pub fn broadcast_data(&mut self, data: &str) {
        let msg = Message::Data(data.into());
        for addr in self.peers.clone() {
            self.send_msg(msg.clone(), addr);
        }
    }

    fn wait(&mut self) {
        loop {
            let (msg, src_addr) = self.recv_msg();
            self.handle_msg(msg, &src_addr);
        }
    }

    fn recv_msg(&mut self) -> (Message, SocketAddr) {
        let mut buf = [0; 500];
        let (number_of_bytes, src_addr)
            = self.socket.recv_from(&mut buf)
                  .expect("Didn't receive data");
        let msg: Message = deserialize(&buf).expect("Deserialization failure");

        println!("recv({:?}): {:?}", src_addr, msg);

        (msg, src_addr)
    }

    fn send_puncher_msg(&mut self, msg: Message) -> std::io::Result<usize> {
        // copy to avoid conflicting borrow below
        let paddr = self.puncher_addr;
        self.send_msg(msg, &paddr)
    }

    fn send_msg<T: ToSocketAddrs>(&mut self, msg: Message, dest_addr: T) -> std::io::Result<usize> {
        println!("send({:?}): {:?}", dest_addr.to_socket_addrs().unwrap().next(), msg);
        self.socket.send_to(&serialize(&msg, self.limit).unwrap(), dest_addr)
    }

    fn handle_msg(&mut self, msg: Message, src_addr: &SocketAddr) {
        let maybe_reply = match msg {
            Message::Ping(x) => Some(Message::Pong(x)),
            Message::ConnectTo(addr_str) => {
                if *src_addr == self.puncher_addr {
                    self.connect_to(addr_str.as_str());
                }
                None
            }
            _ => None
        };
        if let Some(reply) = maybe_reply {
            println!("send({:?}): {:?}", src_addr, reply);
            self.socket.send_to(&serialize(&reply, self.limit).unwrap(), src_addr)
                       .expect("reply failed");
        }
    }

    fn connect_to<T: ToSocketAddrs>(&mut self, addr: T) {
        let mut rng = rand::thread_rng();
        let ping = rng.gen::<u32>();
        self.send_msg(Message::Ping(ping), addr)
            .expect("couldn't send ping msg");
        loop {
            let (msg, src_addr) = self.recv_msg();
            match msg {
                Message::Pong(x) if x == ping => {
                    println!("connected to {:?}", src_addr);
                    self.peers.push(src_addr);
                    return;
                },
                other => self.handle_msg(other, &src_addr)
            }
        }
    }

}

fn main() {
    let matches = App::new("sixpunch-client")
        .about("host/client application")
        .subcommand(SubCommand::with_name("host")
                    .about("host mode"))
        .subcommand(SubCommand::with_name("connect")
                    .about("connect to host")
                    .arg(Arg::with_name("ID")
                         .help("host id")
                         .required(true)
                         .index(1)))
        .get_matches();

    match matches.subcommand() {
        ("host", Some(sub_m)) => {
            let mut host = Client::new("[::]:11122", "[::1]:11121");
            let id = host.register_host();
            println!("id: {}", id);
            host.wait();
        },
        ("connect", Some(sub_m)) => {
            let id = u64::from_str_radix(sub_m.value_of("ID").unwrap(), 10)
                .expect("ID must be an integer");
            let mut client = Client::new("[::]:11126", "[::1]:11121");
            client.connect_to_host(id);
            loop {
                let mut input = String::new();
                print!("> "); io::stdout().flush();
                io::stdin().read_line(&mut input).expect("stdin read failed");
                client.broadcast_data(&input);
            }
        },
        _ => assert!(false)
    }
}
