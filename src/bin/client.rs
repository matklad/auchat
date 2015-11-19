extern crate rustc_serialize;
extern crate docopt;
extern crate byteorder;
extern crate chat;

use std::net;
use std::io::{self, BufRead, Write, Read};
use std::str::FromStr;
use std::thread;

use byteorder::{ReadBytesExt, LittleEndian};

use chat::post::Post;


const USAGE: &'static str = "
auchat

Usage:
  auchat [--addr=<host:port>] [--login=<login>]
  chat (-h | --help)

Options:
  --addr=<host:port>          Port to listen [default: 0.0.0.0:20053]
  --login=<login>             Login [default: anonymous]
  -h, --help                  Show this screen.
";


#[derive(Debug, RustcDecodable)]
struct Args {
    flag_addr: String,
    flag_login: String,
}


fn main() {
    let args: Args = docopt::Docopt::new(USAGE)
    .and_then(|d| d.options_first(true).decode())
    .unwrap_or_else(|e| e.exit());

    let Args {flag_addr: addr, flag_login: login} = args;
    let addr: net::SocketAddr = FromStr::from_str(&addr)
    .ok().expect(&format!("Failed to parse host:port string: {}", addr));

    let sock = match net::TcpStream::connect(&addr) {
        Ok(sock) => sock,
        Err(e) => {
            println!("Failed to connect to {}: {}", addr, e);
            return;
        }
    };

    let sock2 = match sock.try_clone() {
        Ok(sock) => sock,
        Err(e) => {
            println!("Cant write and read socket simulaneously: {}", e);
            return;
        }
    };

    println!("Connecting to {}", addr);
    thread::spawn(move || {
        writer(sock2, login)
    });

    thread::spawn(move || {
        reader(sock)
    }).join().unwrap();;

}

fn reader(mut sock: net::TcpStream) {

    loop {
        let msg_len = match sock.read_u32::<LittleEndian>() {
            Ok(len) => len as usize,
            Err(e) => {
                println!("Failed to receive a message: {}", e);
                break;
            }
        };

        let mut buf = vec![0; msg_len];
        if let Err(e) = read_exact(&mut sock, &mut buf) {
            println!("Failed to receive a message: {}", e);
            break;
        }

        let post = match Post::from_bytes(&buf) {
            Ok(post) => Post::from(post),
            Err(e) => {
                println!("Failed to decode a message: {}", e);
                break;
            }
        };
        let (author, lines) = post.take();
        println!("{}: {}", author, lines.join("\n"));
    }
}

fn writer(mut sock: net::TcpStream, login: String) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                let message = Post::new(login.clone(), vec![line]);
                if let Err(e) = write_message(&mut sock, message) {
                    println!("Failed to deliver the message: {}", e);
                    break;
                }
            },
            Err(e) => println!("Error reading line: {}", e),
        }
    }
}

fn write_message(sock: &mut net::TcpStream, msg: Post) -> io::Result<()> {
    try!(sock.write_all(&msg.into_bytes()));
    Ok(())
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


