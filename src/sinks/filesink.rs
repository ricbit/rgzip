use std::fs::File;
use std::io::Write;
use errors::{GzipResult, GzipError};
use sinks::bytesink::{ByteSink, ByteSinkProvider};

pub struct FileSink {
    file: File,
}

impl FileSink {
    pub fn provider(name: String) -> ByteSinkProvider {
        Box::new(move || {
            let file = File::create(&name).or(Err(GzipError::CantCreateFile))?;
            Ok(Box::new(FileSink{ file }))
        })
    }
}

impl ByteSink for FileSink {
    fn put_u8(&mut self, data: u8) -> GzipResult<()> {
        match self.file.write(&[data]) {
            Ok(0) | Err(_) => Err(GzipError::CantWriteFile),
            _ => Ok(())
        }
    }

    fn put_data(&mut self, data: &[u8]) -> GzipResult<()> {
        match self.file.write(data) {
            Ok(0) | Err(_) => Err(GzipError::CantWriteFile),
            _ => Ok(())
        }
    }
}

