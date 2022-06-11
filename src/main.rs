fn main() {
    let kb = hhkbfit::get_dev();

    println!("dips: {:?}", kb.dips());

    println!("mode: {:?}", kb.mode());

    println!("info: {:#?}", kb.info());
}
