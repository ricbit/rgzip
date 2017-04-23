// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

use std::fmt;

pub type GzipResult<T> = Result<T, GzipError>;

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
    DeflateModeNotSupported,
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
            DeflateModeNotSupported => "Deflate mode not supported yet",
        };
        write!(f, "{}", error)
    }
}


