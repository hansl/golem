extern crate nfc;

use nfc::context;
use nfc::misc;
use std::ptr::{null, null_mut};

fn main() {
    let mut context = context::new();

    if context.is_null() {
        println!("Unable to initialize new NFC context!");
    }

    // Initialize libnfc
    nfc::init(&mut context);

    // Print libnfc version
    println!("libnfc version: {}", misc::version());
    let devices = nfc::list_devices(context, null_mut(), 0);
    eprintln!("Found {} NFC device(s)", devices);
}
