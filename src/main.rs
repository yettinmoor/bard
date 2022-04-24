mod bard;
mod block;
mod client;
mod server;

use std::{env, process::exit};

use simple_logger::SimpleLogger;

pub mod bus {
    pub use dbus::blocking::Connection;
    pub const BUS_NAME: &str = "xyz.yettinmoor.bard";
    pub const BARD_OBJECT_NAME: &str = "/xyz/yettinmoor/bard";
}

fn main() {
    SimpleLogger::new().with_local_timestamps().init().unwrap();
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.get(0).map(|s| s.as_str()) {
        Some("init") => server::init(),
        Some("update") => {
            if args.len() > 1 {
                client::update(&args[1..]);
            } else {
                print_usage_and_exit(1);
            }
        }
        Some("update-all") => client::update(&[]),
        Some("restart") => client::restart(),
        Some("help") => print_usage_and_exit(0),
        _ => print_usage_and_exit(1),
    }
}

fn print_usage_and_exit(exit_code: i32) {
    eprintln!("usage:");
    eprintln!("  bard init");
    eprintln!("  bard update [block]+");
    eprintln!("  bard update-all");
    eprintln!("  bard restart");
    eprintln!("  bard help");
    exit(exit_code);
}
