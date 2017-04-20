// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

struct FileSource {
    data : Vec<u8>
}

impl FileSource {
    fn new(name: &String) -> Result<Self, io::Error> {
        let mut source = FileSource{ data: vec![] };        
        let mut file = try!(File::open(name));
        try!(file.read_to_end(&mut source.data));
        Ok(source)
    }
}

trait ByteSource {
    fn get_byte(&self) -> u8;
}

fn main() {
  println!("RGzip 0.1, by Ricardo Bittencourt 2017");

  let args : Vec<String> = env::args().collect();
  if args.len() < 2 {
      println!("Usage: rgzip file.gz");
      return;
  }
  println!("Reading {} ", args[1]);
  let source = FileSource::new(&args[1]);
  match source {
      Ok(_) => println!("Finished"),
      Err(error) => println!("Error: {}", error)
  }
}
