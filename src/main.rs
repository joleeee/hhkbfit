use std::{fs::File, io::Write};

fn main() {
    let kb = hhkbfit::get_dev();

    println!("dips: {:?}", kb.dips());

    println!("mode: {:?}", kb.mode());

    println!("info: {:#?}", kb.info());

    let firmware = kb.dump().unwrap();
    File::create("firmware.bin")
        .unwrap()
        .write_all(&firmware)
        .unwrap();
    File::create("firmware.hex")
        .unwrap()
        .write_all(&hex::encode(&firmware).into_bytes())
        .unwrap();
    println!("firmware written to firmware.{{bin,hex}}");
}
