extern crate clap;
extern crate sixpunch;

use sixpunch::{Client, udp};
use clap::{Arg, App, SubCommand};
use std::io::{self, Write};

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
        .arg(Arg::with_name("puncher")
             .help("address of puncher")
             .long("puncher")
             .required(true)
             .takes_value(true))
        .get_matches();

    let puncher_addr = matches.value_of("puncher").unwrap();

    match matches.subcommand() {
        ("host", Some(sub_m)) => {
            let mut host = udp::Client::new("[::]:11122", &puncher_addr);
            let id = host.register_host();
            println!("id: {}", id);
            host.wait();
        },
        ("connect", Some(sub_m)) => {
            let id = u64::from_str_radix(sub_m.value_of("ID").unwrap(), 10)
                .expect("ID must be an integer");
            let mut client = udp::Client::new("[::]:11126", &puncher_addr);
            if client.connect_to_host(id) {
                loop {
                    let mut input = String::new();
                    print!("> "); io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut input).expect("stdin read failed");
                    client.broadcast_data(&input);
                }
            }
        },
        _ => assert!(false)
    }
}
