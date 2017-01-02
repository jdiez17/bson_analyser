extern crate byteorder;
mod bson;

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::BufReader;

fn main() {
    let filename = match env::args().nth(1) {
        Some(f) => f,
        None => panic!("gimme a filename")
    };
    let f = match File::open(filename) {
        Ok(f) => f,
        Err(_) => panic!("could not open it")
    };
    let mut rdr = BufReader::new(f);
    let mut buf: Vec<u8> = Vec::new();
    let _ = rdr.read_to_end(&mut buf);

    let sz = bson::bson_size(&buf);
    println!("positive sz {:?} negative sz {:?}", 
        sz.get(&vec!["root", "positive"]),
        sz.get(&vec!["root", "negative"]));
}
