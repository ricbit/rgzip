use errors::GzipResult;

pub trait ByteSink {
    fn put_u8(&mut self, data: u8) -> GzipResult<()>;

    fn put_data(&mut self, data: Vec<u8>) -> GzipResult<()> {
        for d in data {
            self.put_u8(d)?;
        }
        Ok(())
    }
}

pub type ByteSinkProvider = Box<Fn() -> GzipResult<Box<ByteSink>> + Send + Sync + 'static>;
