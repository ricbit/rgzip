// Gzip decompressor in Rust
// Ricardo Bittencourt 2017

use std::fmt;

pub type GzipResult<T> = Result<T, GzipError>;

#[derive(Debug)]
pub enum GzipError {
    CantOpenFile,
    CantReadFile,
    CantCreateFile,
    CantWriteFile,
    NotAGzipFile,
    TruncatedFile,
    NotDeflate,
    FEXTRANotSupported,
    FHCRCNotSupported,
    FCOMMENTNotSupported,
    ReservedFlagsNotSupported,
    DeflateModeNotSupported,
    StoredHeaderFailure,
    InternalError,
    InvalidDeflateStream,
}

impl fmt::Display for GzipError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use GzipError::*;
        let error = match *self {
            CantOpenFile => "Can't open file",
            CantReadFile => "Can't read from file",
            CantCreateFile => "Can't create file",
            CantWriteFile => "Can't write to file",
            NotAGzipFile => "Not a Gzip file",
            TruncatedFile => "Truncated file",
            NotDeflate => "Not a deflate stream",
            FEXTRANotSupported => "Header flag FEXTRA not supported yet",
            FHCRCNotSupported => "Header flag FHCRC not supported yet",
            FCOMMENTNotSupported => "Header flag FCOMMENT not supported yet",
            ReservedFlagsNotSupported => "Reserved header flags not supported",
            DeflateModeNotSupported => "Reserved deflate mode not defined yet",
            StoredHeaderFailure => "Error in stored block header",
            InternalError => "Internal decoder error",
            InvalidDeflateStream => "Invalid deflate stream",
        };
        write!(f, "{}", error)
    }
}


