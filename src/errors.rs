// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

use std::fmt;
/*use std::env;
use std::fs::File;
use std::io::prelude::*;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;*/

pub enum GzipError {
    CantOpenFile,
    CantReadFile,
    NotAGzipFile,
    TruncatedFile,
    NotDeflate,
    FEXTRANotSupported,
    FHCRCNotSupported,
    FCOMMENTNotSupported,
    ReservedFlagsNotSupported,
}

impl fmt::Display for GzipError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use GzipError::*;
        let error = match *self {
            CantOpenFile => "Can't open file",
            CantReadFile => "Can't read from file",
            NotAGzipFile => "Not a Gzip file",
            TruncatedFile => "Truncated file",
            NotDeflate => "Not a deflate stream",
            FEXTRANotSupported => "Header flag FEXTRA not supported yet",
            FHCRCNotSupported => "Header flag FHCRC not supported yet",
            FCOMMENTNotSupported => "Header flag FCOMMENT not supported yet",
            ReservedFlagsNotSupported => "Reserved header flags not supported",
        };
        write!(f, "{}", error)
    }
}


