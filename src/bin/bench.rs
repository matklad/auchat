extern crate byteorder;
extern crate chat;
extern crate time;
extern crate simple_parallel;

use std::net;
use std::io::{self, Write, Read};
use std::str::FromStr;

use byteorder::{ReadBytesExt, LittleEndian};

use chat::post::Post;

fn message() -> Post {
    Post::new("matklad".to_string(), vec!["Hello, World! ".to_string()])
}

fn c10k() {
    let addr: net::SocketAddr = FromStr::from_str("0.0.0.0:20053").unwrap();
    let mut socks = Vec::new();
    let message = message();
    let start = time::precise_time_s();
    let mut pool = simple_parallel::Pool::new(4);
    for n_cons in 0..10_000 {
        if n_cons % 100 == 0 {
            println!("{} concurrent connections, {:.2} seconds",
                     n_cons, time::precise_time_s() - start);
        }
        let mut sock = net::TcpStream::connect(&addr).unwrap();
        sock.write_all(&message.to_bytes()).unwrap();
        socks.push(sock);
        pool.for_(socks.iter_mut(), |mut sock| {
            let msg_len = sock.read_u32::<LittleEndian>().unwrap() as usize;
            let mut buf = vec![0; msg_len];
            read_exact(&mut sock, &mut buf).unwrap();
        });
    }

    let end = time::precise_time_s();
    let duration = end - start;

    println!("time {:.2} seconds", duration);
}

fn sequential() {
    let addr: net::SocketAddr = FromStr::from_str("0.0.0.0:20053").unwrap();

    let mut sock = net::TcpStream::connect(&addr).unwrap();

    let message = message();
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
    println!("Throughput {:.2} mb/s", mb as f64 / duration);
    println!("Throughput {:.2} requests/s", n_requests as f64 / duration);
}


fn main() {
    c10k();
//    sequential();
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
