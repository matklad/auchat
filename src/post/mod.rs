use std::io::{self, Write};

use byteorder::{LittleEndian, WriteBytesExt};
use protobuf::{self, Message};

mod proto_post;

use self::proto_post::Post as ProtoPost;

pub struct Post {
    pub login: String,
    pub text: String,
}


impl Post {
    pub fn into_bytes(self) -> Vec<u8> {
        let proto = self.into_proto();
        let proto = proto.write_to_bytes().unwrap();

        let mut result = Vec::new();
        result.write_u64::<LittleEndian>(proto.len() as u64).unwrap();
        result.write(&proto).unwrap();
        result
    }

    pub fn from_bytes(bytes: Vec<u8>) -> io::Result<Post> {
        let proto = try!(protobuf::parse_from_bytes::<ProtoPost>(&bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)));
        Ok(Post::from_proto(proto))
    }                           //
    //
    fn into_proto(self) -> ProtoPost { //
        let mut result = ProtoPost::default(); //
        result.set_login(self.login); //
        result.set_text(self.text); //
        result                  //
    }                           //
    //
    fn from_proto(mut post: ProtoPost) -> Post { //
        Post {
            login: post.take_login(),
            text: post.take_text(),
        }
    }

}
