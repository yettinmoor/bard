use crate::bus;
use dbus::Error;
use std::time::Duration;

pub fn update(blocks: &[String]) {
    let conn = bus::Connection::new_session().expect("could not connect to dbus");
    let proxy = conn.with_proxy(
        bus::BUS_NAME,
        bus::BARD_OBJECT_NAME,
        Duration::from_millis(1000),
    );
    let res: Result<(bool, String), Error> = if !blocks.is_empty() {
        proxy.method_call(bus::BUS_NAME, "update", (blocks,))
    } else {
        proxy.method_call(bus::BUS_NAME, "update_all", ())
    };

    match res {
        Ok((ok, reply)) => {
            eprint!("{}", reply);
            if ok {
                let (bar,): (String,) = proxy.method_call(bus::BUS_NAME, "draw_bar", ()).unwrap();
                eprintln!("bard: `{}`", bar);
            }
        }
        Err(_) => {
            eprintln!("error: could not find running bard instance");
        }
    }
}

pub fn restart() {
    let conn = bus::Connection::new_session().expect("could not connect to dbus");
    let proxy = conn.with_proxy(
        bus::BUS_NAME,
        bus::BARD_OBJECT_NAME,
        Duration::from_millis(1000),
    );
    let _: () = proxy
        .method_call(bus::BUS_NAME, "restart", ())
        .expect("could not find bard");
}
