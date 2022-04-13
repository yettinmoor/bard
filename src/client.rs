use crate::bus;
use std::time::Duration;

pub fn update(blocks: &[String]) {
    let conn = bus::Connection::new_session().expect("could not connect to dbus");
    let proxy = conn.with_proxy(
        bus::BUS_NAME,
        bus::BARD_OBJECT_NAME,
        Duration::from_millis(1000),
    );
    let reply: String = if !blocks.is_empty() {
        let (s,): (String,) = proxy
            .method_call(bus::BUS_NAME, "update", (blocks,))
            .expect("could not find bard");
        s
    } else {
        let (s,) = proxy
            .method_call(bus::BUS_NAME, "update_all", ())
            .expect("could not find bard");
        s
    };
    eprint!("{}", reply);

    let (bar,): (String,) = proxy.method_call(bus::BUS_NAME, "draw_bar", ()).unwrap();
    eprintln!("bard: `{}`", bar);
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
