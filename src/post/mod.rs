use std::io;

use protobuf::{self, Message};

use mio::buf::{ByteBuf, Buf};

mod message;

pub use self::message::Message as ProtoMessage;


impl ProtoMessage {
    pub fn command(&self) -> Option<String> {
        let text = self.get_text();
        let preffix = "command ";
        if text.len() > 0 && text[0].starts_with(preffix) {
            Some(text[0][preffix.len()..].to_string())
        } else {
            None
        }
    }

    pub fn from_result(result: String) -> ProtoMessage {
        let mut proto = ProtoMessage::default();
        proto.set_author("nobody".to_owned());
        proto.set_text(protobuf::RepeatedField::from_vec(vec![result]));
        proto
    }
}

#[derive(Debug, Clone)]
pub struct Post(ProtoMessage);


impl Post {
    pub fn new(author: String, text: Vec<String>) -> Post{
        let mut proto = ProtoMessage::default();
        proto.set_author(author);
        proto.set_text(protobuf::RepeatedField::from_vec(text));
        Post(proto)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let size = self.0.compute_size();
        let mut buf = ByteBuf::mut_with_capacity((size + 5) as usize);
        self.0.write_length_delimited_to_writer(&mut buf).unwrap();
        buf.flip().bytes().iter().map(|&i| i).collect()
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
