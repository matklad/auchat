use std::io;
use std::mem::swap;

use mio::buf::{Buf, ByteBuf};

use byteorder::{ReadBytesExt, LittleEndian};


pub struct Chunker {
    is_reading_length: bool,
    len_buffer: Vec<u8>,
    msg_len: usize,
    msg_buffer: Vec<u8>,
}

impl Chunker {
    pub fn new() -> Chunker {
        Chunker {
            is_reading_length: true,
            len_buffer: Vec::new(),
            msg_len: 0,
            msg_buffer: Vec::new(),
        }
    }

    pub fn feed(&mut self, buf: ByteBuf) -> Vec<Vec<u8>> {
        let mut result = Vec::new();
        for &byte in buf.bytes() {
            if self.is_reading_length {

                self.len_buffer.push(byte);

                if self.len_buffer.len() == 4 {
                    let mut len_buffer = Vec::new();
                    swap(&mut self.len_buffer, &mut len_buffer);
                    self.msg_len = io::Cursor::new(len_buffer).read_u32::<LittleEndian>().unwrap() as usize;
                    self.is_reading_length = false
                }

            } else {

                self.msg_buffer.push(byte);

                if self.msg_buffer.len() == self.msg_len {
                    let mut msg_buffer = Vec::new();
                    swap(&mut self.msg_buffer, &mut msg_buffer);
                    result.push(msg_buffer);
                    self.is_reading_length = true
                }
            }
        }
        result
    }

}
