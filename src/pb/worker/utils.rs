use std::io;


use mio::buf::ByteBuf;
use protobuf;
use byteorder::{LittleEndian, WriteBytesExt};


pub fn to_buf<P: protobuf::Message>(msg: &P) -> ByteBuf {
    let size = msg.compute_size();
    let mut buf = ByteBuf::mut_with_capacity((size + 4) as usize);

    buf.write_u32::<LittleEndian>(size).unwrap();
    msg.write_to_writer(&mut buf).unwrap();
    buf.flip()
}

pub fn from_bytes<P: protobuf::Message + protobuf::MessageStatic>(bytes: &[u8]) -> io::Result<P> {
    protobuf::parse_from_bytes::<P>(bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

