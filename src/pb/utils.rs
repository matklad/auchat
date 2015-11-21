use std::io::{self, Write};

use protobuf;
use byteorder::{LittleEndian, WriteBytesExt};


pub fn to_bytes<P: protobuf::Message>(msg: &P) -> Vec<u8> {
    let proto = msg.write_to_bytes().unwrap();
    let mut result = Vec::new();
    result.write_u32::<LittleEndian>(proto.len() as u32).unwrap();
    result.write(&proto).unwrap();
    result
}

pub fn from_bytes<P: protobuf::Message + protobuf::MessageStatic>(bytes: &[u8]) -> io::Result<P> {
    protobuf::parse_from_bytes::<P>(bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

