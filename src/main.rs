mod api;
mod database;
mod handler;
mod http_utils;

use ctru::prelude::*;
use database::DatabaseHolder;
use handler::Handler;

fn main() {
    ctru::applets::error::set_panic_hook(true);

    let gfx = Gfx::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let apt = Apt::new().unwrap();
    let soc = Soc::new().unwrap();

    let mut db = DatabaseHolder::new();
    let mut handler = Handler::new();

    {
        let _bottom_console = Console::new(gfx.bottom_screen.borrow_mut());
        println!("Serving at {}/\n", soc.host_address());
        println!("Press Start to exit");
    }

    let _top_console = Console::new(gfx.top_screen.borrow_mut());
    while apt.main_loop() {
        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        };

        handler.step(&mut db.db);
        db.step();

        gfx.wait_for_vblank();
    }
}
