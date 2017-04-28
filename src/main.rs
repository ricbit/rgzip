// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

extern crate time;
extern crate encoding;

mod errors;
mod sources;
mod sinks;
mod buffers;
mod blocks;

use std::env;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;
use errors::{GzipResult, GzipError};
use sources::bytesource::ByteSource;
use sources::vecsource::VecSource;
use sources::bitsource::BitSource;
use sources::bitadapter::BitAdapter;
use sinks::filesink::FileSink;
use blocks::stored::BlockStored;
use blocks::fixed::BlockFixed;
use blocks::dynamic::BlockDynamic;
use buffers::outputbuffer::OutputBuffer;
use buffers::inmemory::InMemoryBuffer;


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

struct GzipDecoder<'a, T: 'a + ByteSource, U: 'a + OutputBuffer> {
    input: &'a mut T,
    output: &'a mut U,
    header: GzipHeader
}

impl<'a, T: ByteSource, U: OutputBuffer> GzipDecoder<'a, T, U> {
    fn decode(input : &mut T, output: &mut U) -> GzipResult<()> {
        let mut gzip = GzipDecoder {
            input: input,
            output: output,
            header: GzipHeader::default()
        };
        gzip.decode_header()?;
        gzip.decode_deflate()?;
        Ok(())
    }

    fn decode_deflate<'b>(&mut self) -> GzipResult<()> {
        let mut bits = BitAdapter::new(self.input);
        for i in 1.. {
            let header = BlockHeader{
                BFINAL: bits.get_bit()? as u8,
                BTYPE: bits.get_bits_rev(2)? as u8,
            };
            println!("Block {} is final: {}", i, header.BFINAL > 0);
            try!(match header.BTYPE {
                0 => BlockStored::new(&mut bits, self.output).decode(),
                1 => BlockFixed::new(&mut bits, self.output).decode(),
                2 => BlockDynamic::new(&mut bits, self.output).decode(),
                _ => Err(GzipError::DeflateModeNotSupported),
            });
            if header.BFINAL > 0 {
                break;
            }
        }
        Ok(())
    }

    fn decode_header(&mut self) -> GzipResult<()> {
        use GzipHeaderFlags::*;

        self.header.ID1 = self.input.get_u8()?;
        self.header.ID2 = self.input.get_u8()?;
        if self.header.ID1 != 31 || self.header.ID2 != 139 {
            return Err(GzipError::NotAGzipFile);
        }

        self.header.CM = self.input.get_u8()?;
        if self.header.CM != 8 {
            return Err(GzipError::NotDeflate);
        }

        self.header.FLG = self.input.get_u8()?;
        println!("File type is {}",
            if self.header.FLG & (FTEXT as u8) > 0 {"ASCII"} else {"Binary"});
        if self.header.FLG & (FEXTRA as u8) > 0 {
            return Err(GzipError::FEXTRANotSupported);
        }
        if self.header.FLG & (FHCRC as u8) > 0 {
            return Err(GzipError::FHCRCNotSupported);
        }
        if self.header.FLG & (FCOMMENT as u8) > 0 {
            return Err(GzipError::FCOMMENTNotSupported);
        }
        if self.header.FLG >= 0x20 {
            return Err(GzipError::ReservedFlagsNotSupported);
        }

        self.header.MTIME = self.input.get_u32()?;
        if self.header.MTIME > 0 {
            let timespec = time::Timespec::new(self.header.MTIME as i64, 0);
            let tm = time::at_utc(timespec);
            if let Ok(date) = time::strftime("%F %T", &tm) {
                println!("Date: {}", date);
            }
        }

        self.header.XFL = self.input.get_u8()?;

        self.header.OS = self.input.get_u8()?;
        println!("Operating System: {}", self.translate_os());

        if self.header.FLG & (FNAME as u8) > 0 {
            let mut iso_8859_1 : Vec<u8> = vec![];
            loop {
                let c = self.input.get_u8()?;
                if c == 0 {
                    break;
                }
                iso_8859_1.push(c);
            }
            let utf8 = ISO_8859_1.decode(&iso_8859_1, DecoderTrap::Strict);
            if let Ok(name) = utf8 {
                println!("Original filename: {}", name);
                self.header.original_name = Some(name);
            }
        }
        return Ok(());
    }

    fn translate_os(&self) -> &'static str {
        match self.header.OS {
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

fn read_gzip<'a>(input: &'a String, output: &'a String)
    -> GzipResult<()> {
    let mut source = VecSource::from_file(input)?;
    let mut sink = FileSink::new(output)?;
    let mut buffer = InMemoryBuffer::new(&mut sink);
    GzipDecoder::decode(&mut source, &mut buffer)
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
