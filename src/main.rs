#![feature(thread_id_value)]
#![feature(duration_constructors)]
mod api;
mod database;
mod handler;
mod http_utils;

use std::sync::{Arc, Mutex};

use ctru::prelude::*;
use database::Database;
use handler::Handler;

const WORKER_COUNT: usize = 3;

fn main() {
    ctru::applets::error::set_panic_hook(true);

    let gfx = Gfx::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let apt = Apt::new().unwrap();
    let soc = Soc::new().unwrap();

    {
        let _bottom_console = Console::new(gfx.bottom_screen.borrow_mut());
        println!("Serving at {}/\n", soc.host_address());
        println!("Running {} workers", WORKER_COUNT);
        println!("Press Start to exit");
    }

    let _top_console = Console::new(gfx.top_screen.borrow_mut());
    let db = Arc::new(Mutex::new(Database::new()));
    let mut handler = Handler::new(db.clone(), WORKER_COUNT);


    while apt.main_loop() {
        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        };

        handler.step();
        db.lock().unwrap().step();

        gfx.wait_for_vblank();
    }
    println!("Attempting to stop workers");
    handler.stop_workers();
}
