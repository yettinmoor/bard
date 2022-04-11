mod bard;
mod block;

use crate::bard::Bard;

use std::{env, process::exit, time::Duration};

use dbus::blocking::Connection;
use dbus_crossroads::{Crossroads, IfaceBuilder};

const BUS_NAME: &str = "xyz.yettinmoor.bard";
const BARD_OBJECT_NAME: &str = "/xyz/yettinmoor/bard";

fn init_server() {
    let bard = Bard::init(None);

    let conn = Connection::new_session().expect("could not connect to dbus");
    conn.request_name(BUS_NAME, false, false, true)
        .expect("could not request name");

    let mut cr = Crossroads::new();
    let iface_tok = cr.register(BUS_NAME, move |b: &mut IfaceBuilder<Bard>| {
        b.method(
            "update",
            ("blocks",),
            ("reply",),
            |_, bard, (blocks,): (Vec<String>,)| Ok((bard.update(&blocks),)),
        );
        b.method("update_all", (), ("reply",), |_, bard, _: ()| {
            Ok((bard.update_all(),))
        });
        b.method("draw_bar", (), ("reply",), |_, bard, _: ()| {
            Ok((bard.draw_bar(),))
        });
        b.method("restart", (), (), |_, bard, _: ()| {
            bard.restart();
            Ok(())
        });
    });
    cr.insert(BARD_OBJECT_NAME, &[iface_tok], bard);
    cr.serve(&conn).unwrap();
}

fn remote_update(blocks: &[String]) {
    let conn = Connection::new_session().expect("could not connect to dbus");
    let proxy = conn.with_proxy(BUS_NAME, BARD_OBJECT_NAME, Duration::from_millis(1000));
    let reply: String = if !blocks.is_empty() {
        let (s,): (String,) = proxy
            .method_call(BUS_NAME, "update", (blocks,))
            .expect("could not find bard");
        s
    } else {
        let (s,) = proxy
            .method_call(BUS_NAME, "update_all", ())
            .expect("could not find bard");
        s
    };
    eprint!("{}", reply);

    let (bar,): (String,) = proxy.method_call(BUS_NAME, "draw_bar", ()).unwrap();
    eprintln!("bard: `{}`", bar);
}

fn remote_restart() {
    let conn = Connection::new_session().expect("could not connect to dbus");
    let proxy = conn.with_proxy(BUS_NAME, BARD_OBJECT_NAME, Duration::from_millis(1000));
    let _: () = proxy
        .method_call(BUS_NAME, "restart", ())
        .expect("could not find bard");
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.get(0).map(|s| s.as_str()) {
        Some("init") => init_server(),
        Some("update") => {
            if args.len() > 1 {
                remote_update(&args[1..]);
            } else {
                print_usage_and_exit(true);
            }
        }
        Some("update-all") => remote_update(&[]),
        Some("restart") => remote_restart(),
        Some("help") => print_usage_and_exit(false),
        _ => print_usage_and_exit(true),
    }
}

fn print_usage_and_exit(error: bool) {
    eprintln!("usage:");
    eprintln!("  bard init");
    eprintln!("  bard update [block]+");
    eprintln!("  bard update-all");
    eprintln!("  bard help");
    exit(if error { 1 } else { 0 });
}
