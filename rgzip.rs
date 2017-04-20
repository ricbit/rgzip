// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

trait ByteSource {
    fn get_byte(&mut self) -> Option<u8>;
}

struct VecSource {
    data : Vec<u8>,
    pos: usize
}

impl ByteSource for VecSource {
    fn get_byte(&mut self) -> Option<u8> {
        let ans = if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        };
        self.pos += 1;
        ans
    }
}

enum GzipError {
    CantOpenFile,
    CantReadFile
}

impl fmt::Display for GzipError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use GzipError::*;
        let error = match *self {
            CantOpenFile => "Can't open file",
            CantReadFile => "Can't read from file"
        };
        write!(f, "{}", error)
    }
}

impl VecSource {
    fn from_file(name: &String) -> Result<Self, GzipError> {
        use GzipError::*;
        let mut data: Vec<u8> = vec![];
        let mut file = try!(File::open(name).or(Err(CantOpenFile)));
        try!(file.read_to_end(&mut data).or(Err(CantReadFile)));
        let source = VecSource{ data: data, pos: 0 };
        Ok(source)
    }
}

fn main() {
  println!("RGzip 0.1, by Ricardo Bittencourt 2017");

  let args : Vec<String> = env::args().collect();
  if args.len() < 2 {
      println!("Usage: rgzip file.gz");
      return;
  }
  println!("Reading {} ", args[1]);
  let source = VecSource::from_file(&args[1]);
  match source {
      Ok(_) => println!("Finished"),
      Err(error) => println!("Error: {}", error)
  }
}
