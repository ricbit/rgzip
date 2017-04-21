// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

enum GzipError {
    CantOpenFile,
    CantReadFile,
    NotAGzipFile,
    TruncatedFile,
    NotDeflate
}

impl fmt::Display for GzipError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use GzipError::*;
        let error = match *self {
            CantOpenFile => "Can't open file",
            CantReadFile => "Can't read from file",
            NotAGzipFile => "Not a Gzip file",
            TruncatedFile => "Truncated file",
            NotDeflate => "Not a deflate stream"
        };
        write!(f, "{}", error)
    }
}

trait ByteSource {
    fn get_u8(&mut self) -> Result<u8, GzipError>;

    fn get_u16(&mut self) -> Result<u16, GzipError> {
        let mut ans : u16 = 0;
        for i in 0..2 {
            ans |= (try!(self.get_u8()) as u16) << (8 * i);
        }
        Ok(ans)
    }
    fn get_u32(&mut self) -> Result<u32, GzipError> {
        let mut ans : u32 = 0;
        for i in 0..4 {
            ans |= (try!(self.get_u8()) as u32) << (8 * i);
        }
        Ok(ans)
    }
}

struct VecSource {
    data : Vec<u8>,
    pos: usize
}

impl ByteSource for VecSource {
    fn get_u8(&mut self) -> Result<u8, GzipError> {
        let ans = if self.pos < self.data.len() {
            Ok(self.data[self.pos])
        } else {
            Err(GzipError::TruncatedFile)
        };
        self.pos += 1;
        ans
    }
}

impl VecSource {
    fn from_file(name: &String) -> Result<Self, GzipError> {
        use GzipError::*;
        let mut data = vec![];
        let mut file = try!(File::open(name).or(Err(CantOpenFile)));
        try!(file.read_to_end(&mut data).or(Err(CantReadFile)));
        Ok(VecSource{ data: data, pos: 0 })
    }
}

#[allow(dead_code)]
#[allow(non_snake_case)]
enum GzipHeaderFlags {
    FTEXT = 1,
    FHCRC = 2,
    FEXTRA = 4,
    FNAME = 8,
    FCOMMENT = 16
}

#[derive(Default)]
#[allow(dead_code)]
#[allow(non_snake_case)]
struct Gzip {
    ID1: u8,
    ID2: u8,
    CM: u8,
    FLG: u8,
    MTIME: u32,
    XFL: u8,
    OS: u8
}

impl Gzip {
    fn decode(data : &mut ByteSource) -> Result<Self, GzipError> {
        use GzipHeaderFlags::*;
        let mut gzip = Gzip::default();
        gzip.ID1 = try!(data.get_u8());
        gzip.ID2 = try!(data.get_u8());
        if gzip.ID1 != 31 || gzip.ID2 != 139 {
            return Err(GzipError::NotAGzipFile);
        }
        gzip.CM = try!(data.get_u8());
        if gzip.CM != 8 {
            return Err(GzipError::NotDeflate);
        }
        gzip.FLG = try!(data.get_u8());
        println!("File type is {}", 
            if gzip.FLG & (FTEXT as u8) > 0 {"ASCII"} else {"Binary"});
        gzip.MTIME = try!(data.get_u32());
        if gzip.MTIME > 0 {
            //println("Date: {}
        }
        return Ok(gzip);
    }
}

fn read_gzip(name: &String) -> Result<Gzip, GzipError> {
    let mut source = try!(VecSource::from_file(name));
    Gzip::decode(&mut source)
}

fn main() {
    println!("RGzip 0.1, by Ricardo Bittencourt 2017");

    let args : Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rgzip file.gz");
        return;
    }
    println!("Reading {} ", args[1]);
    match read_gzip(&args[1]) {
        Ok(_) => println!("Finished"),
        Err(error) => println!("Error: {}", error)
    }
}
