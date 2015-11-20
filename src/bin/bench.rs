extern crate byteorder;
extern crate chat;
extern crate time;
extern crate simple_parallel;
extern crate docopt;
extern crate rustc_serialize;


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
        if n_cons % 500 == 0 {
            println!("{} concurrent connections, {:.2} seconds",
                     n_cons, time::precise_time_s() - start);
        }
        let mut sock = net::TcpStream::connect(&addr).unwrap();
        if n_cons + 1 == 10_000 {
            println!("c10k!");
        }
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


const USAGE: &'static str = "
bench

Usage:
  bench [--seq --c10k]
  chat (-h | --help)

Options:
  --seq          Request per second benchmark
  --c10k         10k concurrent connections benchmark
  -h, --help     Show this screen.
";


#[derive(Debug, RustcDecodable)]
struct Args {
    flag_seq: bool,
    flag_c10k: bool,
}



fn main() {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.options_first(true).decode())
        .unwrap_or_else(|e| e.exit());
    if args.flag_seq {
        println!("seq benchmark");
        sequential();
    }
    if args.flag_c10k {
        println!("\n\nc10k benchmark");
        c10k();
    }
    println!("\n\nBenchmarks finished");
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
