use errors::{GzipResult, GzipError};
use buffers::outputbuffer::OutputBuffer;
use sinks::bytesink::{ByteSink, ByteSinkProvider};
use context::VERBOSE;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Receiver, Sender};

enum ChannelData {
    Value(u8),
    Vector(Vec<u8>),
    Window{length: u32, distance: u32},
    Exit
}

struct ReceiverBuffer {
    buffer: Vec<u8>,
    pos: usize,
    size: usize,
    output: Box<ByteSink>,
    rx: Receiver<ChannelData>
}

pub struct ChannelBuffer {
    tx: Sender<ChannelData>,
    handle: Option<JoinHandle<()>>
}

fn forward_data(mut rb: ReceiverBuffer) -> GzipResult<()> {
    loop {
        match rb.rx.recv() {
            Ok(ChannelData::Value(v)) => { 
                rb.put_u8(v)?; },
            Ok(ChannelData::Vector(v)) => { rb.put_data(v)?; },
            Ok(ChannelData::Window{length, distance}) => {
                rb.copy_window(distance, length)?;
            },
            Ok(ChannelData::Exit) => { return Ok(()); },
            _ => panic!("Error in thread receiver")
        }
    }
}

impl ChannelBuffer {
    pub fn new(provider: ByteSinkProvider) -> GzipResult<Self> {
        let (tx, rx) = channel();
        let handle = thread::spawn(move || {
            let output = provider().unwrap();
            let rb = ReceiverBuffer {
                buffer: vec![0; 32768], pos: 0, size: 0,
                output, rx
            };
            forward_data(rb).unwrap();
        });
        Ok(ChannelBuffer{ tx, handle: Some(handle) })
    }
}

impl Drop for ChannelBuffer {
    fn drop(&mut self) {
        self.tx.send(ChannelData::Exit).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}

impl OutputBuffer for ChannelBuffer {
    fn put_u8(&mut self, data: u8) -> GzipResult<()> {
        self.tx
            .send(ChannelData::Value(data))
            .or(Err(GzipError::InternalError))
    }

    fn put_data(&mut self, data: Vec<u8>) -> GzipResult<()> {
        self.tx
            .send(ChannelData::Vector(data))
            .or(Err(GzipError::InternalError))
    }

    fn copy_window(&mut self, distance: u32, length: u32) -> GzipResult<()> {
        self.tx
            .send(ChannelData::Window{length, distance})
            .or(Err(GzipError::InternalError))
    }
}

impl ReceiverBuffer {
    fn put_u8(&mut self, data: u8) -> GzipResult<()> {
        self.buffer[self.pos] = data;
        self.pos = (self.pos + 1) & 32767;
        self.size += 1;
        self.output.put_u8(data)
    }

    fn put_data(&mut self, data: Vec<u8>) -> GzipResult<()> {
        for d in &data {
            self.buffer[self.pos] = *d;
            self.pos = (self.pos + 1) & 32767;
        }
        self.size += data.len();
        self.output.put_data(&data)
    }

    fn copy_window(&mut self, distance: u32, length: u32) -> GzipResult<()> {
        let distance = distance as usize;
        if distance > self.size {
            return Err(GzipError::InvalidDeflateStream);
        }
        let index : usize = self.pos + 32768 - distance;
        verbose!(2, "window char: ");
        for i in 0..length {
            let data = self.buffer[(index + i as usize) & 32767];
            verbose!(2, "-- {}", data as u8 as char);
            self.buffer[self.pos] = data;
            self.pos = (self.pos + 1) & 32767;
            self.output.put_u8(data)?;
        }
        self.size += length as usize;
        Ok(())
    }
}

