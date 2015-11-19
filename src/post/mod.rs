use std::io::{self, Write};

use byteorder::{LittleEndian, WriteBytesExt};
use protobuf::{self, Message};

mod message;

use self::message::Message as ProtoMessage;

#[derive(Debug, Clone)]
pub struct Post(ProtoMessage);


impl Post {
    pub fn new(author: String, text: Vec<String>) -> Post{
        let mut proto = ProtoMessage::default();
        proto.set_author(author);
        proto.set_text(protobuf::RepeatedField::from_vec(text));
        Post(proto)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let Post(proto) = self;
        let proto = proto.write_to_bytes().unwrap();
        let mut result = Vec::new();
        result.write_u32::<LittleEndian>(proto.len() as u32).unwrap();
        result.write(&proto).unwrap();
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Post> {
        let proto = try!(protobuf::parse_from_bytes::<ProtoMessage>(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)));
        Ok(Post(proto))
    }

    pub fn command(&self) -> Option<String> {
        let text = self.0.get_text();
        let preffix = "command ";
        if text.len() > 0 && text[0].starts_with(preffix) {
            Some(text[0][preffix.len()..].to_string())
        } else {
            None
        }
    }

    pub fn take(mut self) -> (String, Vec<String>) {
        (self.0.take_author(), self.0.take_text().into_vec())
    }
}
