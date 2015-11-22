use protobuf::{self, Message};

use mio::buf::{ByteBuf, Buf};

mod message;

pub use self::message::Message as Post;

impl Post {
    pub fn command(&self) -> Option<String> {
        let text = self.get_text();
        let preffix = "command ";
        if text.len() > 0 && text[0].starts_with(preffix) {
            Some(text[0][preffix.len()..].to_string())
        } else {
            None
        }
    }

    pub fn from_result(result: String) -> Post {
        let mut proto = Post::default();
        proto.set_author("nobody".to_owned());
        proto.set_text(protobuf::RepeatedField::from_vec(vec![result]));
        proto
    }

    pub fn from_text(author: String, text: Vec<String>) -> Post {
        let mut proto = Post::default();
        proto.set_author(author);
        proto.set_text(protobuf::RepeatedField::from_vec(text));
        proto
    }

    pub fn take(mut self) -> (String, Vec<String>) {
        (self.take_author(), self.take_text().into_vec())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let size = self.compute_size();
        let mut buf = ByteBuf::mut_with_capacity((size + 5) as usize);
        self.write_length_delimited_to_writer(&mut buf).unwrap();
        buf.flip().bytes().iter().map(|&i| i).collect()
    }

}
