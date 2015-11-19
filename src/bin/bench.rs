extern crate byteorder;
extern crate chat;
extern crate time;

use std::net;
use std::io::{self, Write, Read};
use std::str::FromStr;

use byteorder::{ReadBytesExt, LittleEndian};

use chat::post::Post;




fn main() {
    let addr: net::SocketAddr = FromStr::from_str("0.0.0.0:20053").unwrap();

    let mut sock = match net::TcpStream::connect(&addr) {
        Ok(sock) => sock,
        Err(e) => {
            println!("Failed to connect to {}: {}", addr, e);
            return;
        }
    };

    let message = Post::new("matklad".to_string(),
                            vec!["Hello, World! ".to_string()]);
    let mut bytes_writen = 0;
    let mut bytes_recieved = 0;
    let start = time::precise_time_s();
    println!("message len {}", message.to_bytes().len());
    let n_requests = 100_000;
    for _ in 0..n_requests {
        let message = message.to_bytes();
        bytes_writen += message.len();
        sock.write_all(&message).unwrap();
        let msg_len = sock.read_u32::<LittleEndian>().unwrap() as usize;
        bytes_recieved += 4 + msg_len;
        let mut buf = vec![0; msg_len];
        read_exact(&mut sock, &mut buf).unwrap();
    }
    if bytes_recieved != bytes_writen {
        panic!("broken bench!");
    }
    let end = time::precise_time_s();
    let duration = end - start;
    let mb = bytes_writen / 1024 / 1024;
    println!("Written {} kilobytes", mb);
    println!("time {:.2} seconds", duration);
    println!("Thoruput {:.2} mb/s", mb as f64 / duration);
    println!("Thoruput {:.2} rps", n_requests as f64 / duration);
}


fn read_exact(sock: &mut net::TcpStream, mut buf: &mut [u8]) -> io::Result<()> {
    while !buf.is_empty() {
        match sock.read(buf) {
            Ok(0) => break,
            Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    if !buf.is_empty() {
        Err(io::Error::new(io::ErrorKind::InvalidData,
                           "failed to fill whole buffer"))
    } else {
        Ok(())
    }
}
