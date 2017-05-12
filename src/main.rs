// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

extern crate time;
extern crate encoding;
extern crate getopts;
extern crate memmap;

#[macro_use]
mod context;
mod errors;
mod sources;
mod sinks;
mod buffers;
mod blocks;

use std::env;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;
use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use sources::bytesource::ByteSource;
use sources::vecsource::VecSource;
use sources::bitadapter::BitAdapter;
use sources::widesource::WideSource;
use sources::wideadapter::WideAdapter;
use sources::buffersource::BufferSource;
use sources::vecbufsource::VecBufSource;
use sources::mapsource::MapSource;
use sinks::bytesink::ByteSinkProvider;
use sinks::filesink::FileSink;
use sinks::filebufsink::FileBufSink;
use blocks::stored::BlockStored;
use blocks::fixed::BlockFixed;
use blocks::dynamic::BlockDynamic;
use buffers::outputbuffer::OutputBuffer;
use buffers::inmemory::InMemoryBuffer;
use buffers::circular::CircularBuffer;
use buffers::channel::ChannelBuffer;
use buffers::copy::CopyBuffer;
use getopts::Options;
use context::{VERBOSE, SINK, SOURCE, BUFFER, ADAPTER};

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

struct GzipDecoder {
    input: Box<ByteSource>,
    output: Box<OutputBuffer>,
    header: GzipHeader
}

impl GzipDecoder {
    fn decode(input : Box<ByteSource>, output: Box<OutputBuffer>)
        -> GzipResult<()> {

        let mut gzip = GzipDecoder {
            input: input,
            output: output,
            header: GzipHeader::default()
        };
        gzip.decode_header()?;
        gzip.decode_deflate()?;
        Ok(())
    }

    fn decode_deflate(&mut self) -> GzipResult<()> {
        let mut bits : Box<BitSource> = match get_context!(ADAPTER) {
            0 => Box::new(BitAdapter::new(self.input.as_mut())),
            1 => Box::new(WideAdapter::new(self.input.as_mut())),
            _ => return Err(GzipError::InternalError)
        };
        for i in 1.. {
            let header = BlockHeader{
                BFINAL: bits.get_bit()? as u8,
                BTYPE: bits.get_bits_rev(2)? as u8,
            };
            verbose!(1, "Block {} is final: {}", i, header.BFINAL > 0);
            let mut output = self.output.as_mut();
            try!(match header.BTYPE {
                0 => BlockStored::new(bits.as_mut(), output).decode(),
                1 => BlockFixed::new(bits.as_mut(), output).decode(),
                2 => BlockDynamic::new(bits.as_mut(), output).decode(),
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
        verbose!(1, "File type is {}",
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
                verbose!(1, "Date: {}", date);
            }
        }

        self.header.XFL = self.input.get_u8()?;

        self.header.OS = self.input.get_u8()?;
        verbose!(1, "Operating System: {}", self.translate_os());

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
                verbose!(1, "Original filename: {}", name);
                self.header.original_name = Some(name);
            }
        }
        Ok(())
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

fn choose_sink(output: String) -> GzipResult<ByteSinkProvider> {
    match get_context!(SINK) {
        0 => Ok(FileSink::provider(output)),
        1 => Ok(FileBufSink::provider(output)),
        _ => Err(GzipError::InternalError)
    }
}

fn choose_buffer(sink: ByteSinkProvider) -> GzipResult<Box<OutputBuffer>> {
    match get_context!(BUFFER) {
        0 => Ok(Box::new(InMemoryBuffer::new(sink)?)),
        1 => Ok(Box::new(CircularBuffer::new(sink)?)),
        2 => Ok(Box::new(ChannelBuffer::new(sink)?)),
        3 => Ok(Box::new(CopyBuffer::new(sink)?)),
        _ => Err(GzipError::InternalError)
    }
}

fn choose_source(input: &str) -> GzipResult<Box<ByteSource>> {
    match get_context!(SOURCE) {
        0 => Ok(Box::new(VecSource::from_file(input)?)),
        1 => Ok(Box::new(BufferSource::from_file(input)?)),
        2 => Ok(Box::new(VecBufSource::from_file(input)?)),
        3 => Ok(Box::new(WideSource::from_file(input)?)),
        4 => Ok(Box::new(MapSource::from_file(input)?)),
        _ => Err(GzipError::InternalError)
    }
}

fn read_gzip(input: &str, output: String) -> GzipResult<()> {
    let sink = choose_sink(output)?;
    let buffer = choose_buffer(sink)?;
    let source  = choose_source(input)?;
    GzipDecoder::decode(source, buffer)
}

const USAGE : &str = "Usage: rgzip [flags] input output";

macro_rules! parse_int_argument {
    ($matches: expr, $arg: expr, $limit: expr, $msg: expr, $var: ident) => {
        if let Some(n) = $matches.opt_str($arg) {
            match n.parse::<u8>() {
                Ok(v) if v <= $limit => {
                    unsafe {
                        $var = v;
                    };
                },
                _ => {
                    println!("{}", $msg);
                    return;
                }
            }
        }
    }
}

fn main() {
    println!("rgzip 0.1, by Ricardo Bittencourt 2017");

    let args : Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optopt("v", "verbose", "Verbosity level [0-2]", "v")
        .optopt("s", "source",
                "Source method 0=Vec 1=Buffer 2=VecBuf 3=Wide(def)", "m")
        .optopt("k", "sink", "Sink method 0=File(def) 1=FileBuf", "m")
        .optopt("b", "buffer", 
                "Buffer method 0=InMemory 1=Circular \
                2=Channel 3=Copy(def) 4=Map", "m")
        .optopt("a", "adapter", "Adapter method 0=Bit 1=Wide(def)", "m")
        .optflag("h", "help", "Show help");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage(USAGE));
        return;
    }
    parse_int_argument!(matches, "v", 2, "Invalid verbose level", VERBOSE);
    parse_int_argument!(matches, "k", 1, "Invalid sink method", SINK);
    parse_int_argument!(matches, "s", 4, "Invalid source method", SOURCE);
    parse_int_argument!(matches, "b", 3, "Invalid buffer method", BUFFER);
    parse_int_argument!(matches, "a", 1, "Invalid adapter method", ADAPTER);
    if matches.free.len() < 2 {
        println!("{}", USAGE);
        return;
    }

    let input = &matches.free[0];
    let output = matches.free[1].clone();
    println!("Reading from {}, writing to {}", input, output);
    match read_gzip(input, output) {
        Ok(_) => println!("Finished"),
        Err(error) => println!("Error: {}", error)
    }
}
