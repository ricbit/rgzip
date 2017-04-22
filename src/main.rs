// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

extern crate time;
extern crate encoding;

mod errors;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;
use errors::GzipError;


trait ByteSource {
    fn get_u8(&mut self) -> Result<u8, GzipError>;

    fn get_variable(&mut self, size: u8) -> Result<u32, GzipError> {
        let mut ans : u32 = 0;
        for i in 0..size {
            ans |= (try!(self.get_u8()) as u32) << (8 * i);
        }
        Ok(ans)
    }

    fn get_u16(&mut self) -> Result<u16, GzipError> {
        self.get_variable(2).map(|x| x as u16)
    }

    fn get_u32(&mut self) -> Result<u32, GzipError> {
        self.get_variable(4)
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
    OS: u8,
    original_name: Option<String>,
}

impl Gzip {
    fn decode<T: ByteSource>(data : &mut T) -> Result<Self, GzipError> {
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
        if gzip.FLG & (FEXTRA as u8) > 0 {
            return Err(GzipError::FEXTRANotSupported);
        }
        if gzip.FLG & (FHCRC as u8) > 0 {
            return Err(GzipError::FHCRCNotSupported);
        }
        if gzip.FLG & (FCOMMENT as u8) > 0 {
            return Err(GzipError::FCOMMENTNotSupported);
        }
        if gzip.FLG >= 0x20 {
            return Err(GzipError::ReservedFlagsNotSupported);
        }

        gzip.MTIME = try!(data.get_u32());
        if gzip.MTIME > 0 {
            let timespec = time::Timespec::new(gzip.MTIME as i64, 0);
            let tm = time::at_utc(timespec);
            let display = time::strftime("%F %T", &tm);
            if let Ok(date) = display {
                println!("Date: {}", date);
            }
        }

        gzip.XFL = try!(data.get_u8());

        gzip.OS = try!(data.get_u8());
        println!("Operating System: {}", gzip.translate_os());

        if gzip.FLG & (FNAME as u8) > 0 {
            let mut iso_8859_1 : Vec<u8> = vec![];
            loop {
                let c = try!(data.get_u8());
                if c == 0 {
                    break;
                }
                iso_8859_1.push(c);
            }
            let utf8 = ISO_8859_1.decode(&iso_8859_1, DecoderTrap::Strict);
            if let Some(name) = utf8.ok() {
                println!("Original filename: {}", name);
                gzip.original_name = Some(name);
            }
        }
        return Ok(gzip);
    }

    fn translate_os(&self) -> &'static str {
        match self.OS {
			0 => "FAT filesystem (MS-DOS, OS/2, NT/Win32)",
			1 => "Amiga",
			2 => "VMS (or OpenVMS)",
			3 => "Unix",
			4 => "VM/CMS",
			5 => "Atari TOS",
			6 => "HPFS filesystem (OS/2, NT)",
			7 => "Macintosh",
			8 => "Z-System",
			9 => "CP/M",
			10 => "TOPS-20",
			11 => "NTFS filesystem (NT)",
			12 => "QDOS",
			13 => "Acorn RISCOS",
			_ => "unknown"
        }
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
