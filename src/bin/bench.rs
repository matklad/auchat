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

const USAGE: &'static str = "
bench

Usage:
  bench [--rps --med --large --huge --c10k]
  chat (-h | --help)

Options:
  --rps          Request per second benchmark. Single thread.
  --med          Megabytes per second benchmark. 1k message, single thread.
  --large        Megabytes per second benchmark. 2.7mb message, single thread.
  --huge         Megabytes per second benchmark. 800mb message, single thread.
  --c10k         10k concurrent connections benchmark. Four threads.
  -h, --help     Show this screen.
";


#[derive(Debug, RustcDecodable)]
struct Args {
    flag_rps: bool,
    flag_med: bool,
    flag_large: bool,
    flag_huge: bool,
    flag_c10k: bool,
}



fn main() {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.options_first(true).decode())
        .unwrap_or_else(|e| e.exit());
    if args.flag_rps {
        println!("rps benchmark");
        rps();
    }
    if args.flag_med {
        println!("\n\nmed benchmark");
        med();
    }
    if args.flag_large {
        println!("\n\nlarge benchmark");
        large();
    }
    if args.flag_huge {
        println!("\n\nhuge benchmark");
        huge();
    }
    if args.flag_c10k {
        println!("\n\nc10k benchmark");
        c10k();
    }
    println!("\n\nBenchmarks finished");
}

fn message(message_size: usize) -> Post {
    let text = std::iter::repeat("Hello, World!").take(message_size)
    .collect::<Vec<_>>()
    .join(" ");
    Post::new("matklad".to_string(), vec![text.to_string()])
}

fn c10k() {
    let addr: net::SocketAddr = FromStr::from_str("0.0.0.0:20053").unwrap();
    let mut socks = Vec::new();
    let message = message(1).to_bytes();
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
        sock.write_all(&message).unwrap();
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

fn rps()   { requests(100_000, 1)         }
fn med()   { requests(100_000, 73)        }
fn large() { requests(100    , 200000)    }
fn huge()  { requests(5      , 60000000)  }

fn requests(n_requests: u32, message_size: usize) {
    let addr: net::SocketAddr = FromStr::from_str("0.0.0.0:20053").unwrap();

    let mut sock = net::TcpStream::connect(&addr).unwrap();

    let message = message(message_size);
    let mut bytes_writen = 0;
    let mut bytes_recieved = 0;
    let start = time::precise_time_s();
    let message_len = message.to_bytes().len();
    let message_len_kb = message_len / 1024;
    let message_len_mb = message_len_kb / 1024;
    if message_len_mb == 0 {
        println!("message len {} bytes", message_len);
    } else if message_len_mb < 10 {
        println!("message len {} kb", message_len_kb);
    } else {
        println!("message len {} mb", message_len_mb);
    }
    let message = message.to_bytes();
    for _ in 0..n_requests {
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
    println!("Written {} megabytes", mb);
    println!("time {:.2} seconds", duration);
    println!("Throughput {:.2} mb/s", mb as f64 / duration);
    println!("Throughput {:.2} requests/s", n_requests as f64 / duration);
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
