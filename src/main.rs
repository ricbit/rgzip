// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

extern crate time;
extern crate encoding;

mod errors;
mod sources;
mod sinks;

use std::env;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;
use errors::{GzipResult, GzipError};
use sources::bytesource::ByteSource;
use sources::vecsource::VecSource;
use sources::bitsource::BitSource;
use sources::bitadapter::BitAdapter;
use sinks::bytesink::ByteSink;
use sinks::filesink::FileSink;

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
struct GzipHeader {
    ID1: u8,
    ID2: u8,
    CM: u8,
    FLG: u8,
    MTIME: u32,
    XFL: u8,
    OS: u8,
    original_name: Option<String>,
}

#[allow(non_snake_case)]
struct BlockHeader {
    BFINAL: u8,
    BTYPE: u8,
}

#[allow(non_snake_case)]
struct StoredHeader {
    LEN: u16,
    NLEN: u16
}

impl GzipHeader {
    fn decode<T, U>(data : &mut T, output: &mut U) -> GzipResult<Self>
        where T: ByteSource, U: ByteSink {

        let mut gzip = GzipHeader::default();
        gzip.decode_header(data)?;
        gzip.decode_deflate(data, output)?;
        Ok(gzip)
    }

    fn decode_deflate<T, U>(&mut self, data: &mut T, output: &mut U)
        -> GzipResult<()>
        where T: ByteSource, U: ByteSink {

        let mut bits = BitAdapter::new(data);
        for i in 1.. {
            let header = BlockHeader{
                BFINAL: bits.get_bit()? as u8,
                BTYPE: bits.get_bits_rev(2)? as u8,
            };
            println!("Block {} is final: {}", i, header.BFINAL > 0);
            try!(match header.BTYPE {
                0 => self.decode_stored(&mut bits, output),
                1 => Err(GzipError::StaticHuffmanNotSupported),
                2 => Err(GzipError::DynamicHuffmanNotSupported),
                _ => Err(GzipError::DeflateModeNotSupported),
            });
            if header.BFINAL > 0 {
                break;
            }
        }
        Ok(())
    }

    fn decode_stored<T, U>(&mut self, data: &mut T, output: &mut U)
        -> GzipResult<()>
        where T: BitSource, U: ByteSink {

        let header = StoredHeader{
            LEN: data.get_u16()?,
            NLEN: data.get_u16()?
        };
        if header.LEN ^ header.NLEN != 65535 {
            return Err(GzipError::StoredHeaderFailure);
        }
        for _ in 0..header.LEN {
            let byte = data.get_u8()?;
            output.put_u8(byte)?;
        }
        println!("Stored block, len = {}", header.LEN);
        Ok(())
    }

    fn decode_header<T>(&mut self, data: &mut T) -> GzipResult<()>
        where T: ByteSource {
        use GzipHeaderFlags::*;

        self.ID1 = data.get_u8()?;
        self.ID2 = data.get_u8()?;
        if self.ID1 != 31 || self.ID2 != 139 {
            return Err(GzipError::NotAGzipFile);
        }

        self.CM = data.get_u8()?;
        if self.CM != 8 {
            return Err(GzipError::NotDeflate);
        }

        self.FLG = data.get_u8()?;
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

        self.MTIME = data.get_u32()?;
        if self.MTIME > 0 {
            let timespec = time::Timespec::new(self.MTIME as i64, 0);
            let tm = time::at_utc(timespec);
            if let Ok(date) = time::strftime("%F %T", &tm) {
                println!("Date: {}", date);
            }
        }

        self.XFL = data.get_u8()?;

        self.OS = data.get_u8()?;
        println!("Operating System: {}", self.translate_os());

        if self.FLG & (FNAME as u8) > 0 {
            let mut iso_8859_1 : Vec<u8> = vec![];
            loop {
                let c = data.get_u8()?;
                if c == 0 {
                    break;
                }
                iso_8859_1.push(c);
            }
            let utf8 = ISO_8859_1.decode(&iso_8859_1, DecoderTrap::Strict);
            if let Ok(name) = utf8 {
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

fn read_gzip(input: &String, output: &String) -> GzipResult<GzipHeader> {
    let mut source = VecSource::from_file(input)?;
    let mut sink = FileSink::new(output)?;
    GzipHeader::decode(&mut source, &mut sink)
}

fn main() {
    println!("RGzip 0.1, by Ricardo Bittencourt 2017");

    let args : Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: rgzip input_file output_file");
        return;
    }
    println!("Reading from {}, writing to {}", args[1], args[2]);
    match read_gzip(&args[1], &args[2]) {
        Ok(_) => println!("Finished"),
        Err(error) => println!("Error: {}", error)
    }
}
