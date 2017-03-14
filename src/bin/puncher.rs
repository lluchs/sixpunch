extern crate sixpunch;

use sixpunch::{Puncher, udp};

fn main() {
    let mut puncher = udp::Puncher::new("[::]:11121");
    puncher.run();
}
