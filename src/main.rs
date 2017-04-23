// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

extern crate time;
extern crate encoding;

mod errors;
mod bytesource;
mod vecsource;

use std::env;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;
use errors::GzipError;
use bytesource::ByteSource;
use vecsource::VecSource;

trait BitSource {
    fn get_bit(&mut self) -> Result<u32, GzipError>;

    fn get_bits(&mut self, size: u8) -> Result<u32, GzipError> {
        let mut ans : u32 = 0;
        for _ in 0..size {
            ans = (ans << 1) | try!(self.get_bit());
        }
        Ok(ans)
    }

    fn get_bits_rev(&mut self, size: u8) -> Result<u32, GzipError> {
        let mut ans : u32 = 0;
        for i in 0..size {
            ans |= try!(self.get_bit()) << (1 + i);
        }
        Ok(ans)
    }
}

#[allow(non_snake_case)]
enum GzipHeaderFlags {
    FTEXT = 1,
    FHCRC = 2,
    FEXTRA = 4,
    FNAME = 8,
    FCOMMENT = 16
}

#[derive(Default)]
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
        let mut gzip = Gzip::default();
        try!(gzip.decode_header(data));
        Ok(gzip)
    }

    fn decode_header<T: ByteSource>(&mut self, data : &mut T) 
        -> Result<(), GzipError> {

        use GzipHeaderFlags::*;

        self.ID1 = try!(data.get_u8());
        self.ID2 = try!(data.get_u8());
        if self.ID1 != 31 || self.ID2 != 139 {
            return Err(GzipError::NotAGzipFile);
        }

        self.CM = try!(data.get_u8());
        if self.CM != 8 {
            return Err(GzipError::NotDeflate);
        }

        self.FLG = try!(data.get_u8());
        println!("File type is {}", 
            if self.FLG & (FTEXT as u8) > 0 {"ASCII"} else {"Binary"});
        if self.FLG & (FEXTRA as u8) > 0 {
            return Err(GzipError::FEXTRANotSupported);
        }
        if self.FLG & (FHCRC as u8) > 0 {
            return Err(GzipError::FHCRCNotSupported);
        }
        if self.FLG & (FCOMMENT as u8) > 0 {
            return Err(GzipError::FCOMMENTNotSupported);
        }
        if self.FLG >= 0x20 {
            return Err(GzipError::ReservedFlagsNotSupported);
        }

        self.MTIME = try!(data.get_u32());
        if self.MTIME > 0 {
            let timespec = time::Timespec::new(self.MTIME as i64, 0);
            let tm = time::at_utc(timespec);
            if let Ok(date) = time::strftime("%F %T", &tm) {
                println!("Date: {}", date);
            }
        }

        self.XFL = try!(data.get_u8());

        self.OS = try!(data.get_u8());
        println!("Operating System: {}", self.translate_os());

        if self.FLG & (FNAME as u8) > 0 {
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
                self.original_name = Some(name);
            }
        }
        return Ok(());
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
