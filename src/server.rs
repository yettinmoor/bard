use crate::bard::Bard;
use crate::bus;

use dbus_crossroads::{Crossroads, IfaceBuilder};

pub fn init() {
    let bard = Bard::init(None);

    let conn = bus::Connection::new_session().expect("could not connect to dbus");
    conn.request_name(bus::BUS_NAME, false, false, true)
        .expect("could not request name");

    let mut cr = Crossroads::new();
    let iface_tok = cr.register(bus::BUS_NAME, move |b: &mut IfaceBuilder<Bard>| {
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
    cr.insert(bus::BARD_OBJECT_NAME, &[iface_tok], bard);
    cr.serve(&conn).unwrap();
}
