#[macro_use] extern crate log;
extern crate env_logger;
extern crate mio;
extern crate protobuf;
extern crate byteorder;

use std::net::SocketAddr;

use mio::EventLoop;
use mio::tcp::{TcpListener, TcpStream};

mod shell;
pub mod post;
mod web;

use web::server::Server;
use web::worker::Worker;


#[derive(Debug)]
pub enum Message {
    NewConnection(TcpStream),
    TaskFinished {
        user: String,
        result: Vec<String>,
    },
    NewPost(post::Post),
}


pub fn start_server(addr: SocketAddr, n_workers: usize) {
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
