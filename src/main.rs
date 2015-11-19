#[macro_use] extern crate log;
extern crate env_logger;
extern crate mio;
extern crate rustc_serialize;
extern crate docopt;
extern crate protobuf;
extern crate byteorder;

use std::net::SocketAddr;
use std::str::FromStr;
use std::cmp::max;

use mio::EventLoop;
use mio::tcp::{TcpListener, TcpStream};

mod shell;
mod post;
mod web;

use web::server::Server;
use web::worker::Worker;


#[derive(Debug)]
pub enum Message {
    NewConnection(TcpStream),
    TaskFinished {
        user: String,
        result: String,
    },
    NewPost(post::Post),
}

const USAGE: &'static str = "
Mio chat

Usage:
  chat [--workers=<n_workers> --addr=<host:port>]
  chat (-h | --help)

Options:
  -w, --workers=<n_workers>   Number of worker threads.
  --addr=<host:port>          Port to listen [default: 0.0.0.0:20053]
  -h, --help                  Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_workers: usize,
    flag_addr: String
}


pub fn main() {
    env_logger::init().ok().expect("Failed to init logger");

    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.options_first(true).decode())
        .unwrap_or_else(|e| e.exit());

    let n_workers = max(args.flag_workers, 1);
    let addr = args.flag_addr;

    let addr: SocketAddr = FromStr::from_str(&addr)
        .ok().expect("Failed to parse host:port string");

    let socket = TcpListener::bind(&addr)
        .ok().expect("Failed to bind address");

    let mut event_loop = EventLoop::new()
        .ok().expect("Failed to create event loop");

    let shell = shell::start();
    let workers = Worker::start(n_workers, &shell);

    let mut server = Server::new(socket, workers);
    server.register(&mut event_loop)
        .ok().expect("Failed to register server with event loop");

    println!("Starting server at {}", addr);

    event_loop.run(&mut server)
        .ok().expect("Failed to start event loop");
}
