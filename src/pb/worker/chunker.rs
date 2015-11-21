use std::marker::PhantomData;

use mio::buf::{Buf, ByteBuf};
use protobuf;
use protobuf::stream::WithCodedInputStream;


pub struct Chunker<M: protobuf::MessageStatic> {
    is_reading_length: bool,
    len_buffer: Vec<u8>,
    msg_len: usize,
    msg_buffer: Vec<u8>,
    m: PhantomData<M>,
}

impl<M: protobuf::MessageStatic> Chunker<M> {
    pub fn new() -> Chunker<M> {
        Chunker {
            is_reading_length: true,
            len_buffer: Vec::new(),
            msg_len: 0,
            msg_buffer: Vec::new(),
            m: PhantomData,
        }
    }

    pub fn feed(&mut self, buf: ByteBuf) -> protobuf::ProtobufResult<Vec<M>> {
        let mut result = Vec::new();
        for &byte in buf.bytes() {
            if self.is_reading_length {

                self.len_buffer.push(byte);
                if byte.leading_zeros() > 0 {
                    self.msg_len = try!(self.len_buffer.with_coded_input_stream(|is| {
                        is.read_raw_varint32()
                    } )) as usize;
                    self.len_buffer.truncate(0);
                    self.is_reading_length = false;
                } else if self.len_buffer.len() == 5 {
                    error!("invalid message len");
                }

            } else {

                self.msg_buffer.push(byte);

                if self.msg_buffer.len() == self.msg_len {
                    let msg = try!(protobuf::parse_from_bytes::<M>(&self.msg_buffer));
                    result.push(msg);

                    self.msg_buffer.truncate(0);
                    self.is_reading_length = true;
                }
            }
        }
        Ok(result)
    }

}
