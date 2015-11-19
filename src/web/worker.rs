use std::io;
use std::thread;
use std::sync::mpsc;

use mio::{self, Token, EventSet};
use mio::tcp::TcpStream;
use mio::buf::ByteBuf;
use mio::util::Slab;

use Message;
use shell::Task;
use post::Post;
use super::connection::Connection;


pub struct Worker {
    id: usize,
    token: Token,
    connections: Slab<Connection>,
    shell: mpsc::Sender<Task>,
    peers: Vec<mio::Sender<Message>>,
}

type EventLoop = mio::EventLoop<Worker>;


impl Worker {
    fn new(id: usize, shell: mpsc::Sender<Task>, peers: Vec<mio::Sender<Message>>) -> Worker {
        assert!(id > 0);
        let worker_token = 100 * id - 1;
        Worker {
            id: id,
            token: Token(worker_token),
            connections: Slab::new_starting_at(Token(worker_token + 1), 10_000_000),
            shell: shell,
            peers: peers,
        }
    }

    pub fn start(n_workers: usize, shell: &mpsc::Sender<Task>)
                 -> Vec<mio::Sender<Message>> {
        assert!(n_workers > 0, "Need at least one worker");
        let loops = (0..n_workers)
                .map(|_| EventLoop::new().ok().expect("Failed to crate a worker"))
                .collect::<Vec<_>>();
        let chans = loops.iter().map(|l| l.channel()).collect::<Vec<_>>();

        for (id, mut l) in loops.into_iter().enumerate() {
            let peers = (0..n_workers)
                .filter(|&i| i != id).map(|i| chans[i].clone()).collect();
            let shell = shell.clone();
            thread::spawn(move || {
                l.run(&mut Worker::new(id + 1, shell, peers))
                .ok().expect("Failed to start a worker event loop");
            });

        }

        chans
    }

    fn accept(&mut self, event_loop: &mut EventLoop, sock: TcpStream) {
        match self.connections.insert_with(|token| Connection::new(sock, token)) {
            Some(token) => {
                match self.connections[token].register(event_loop) {
                    Ok(_) => {},
                    Err(e) => {
                        error!("Failed to register connection, {:?}", e);
                        self.connections.remove(token);
                    }
                }
            }
            None => error!("Failed to insert connection into slab")
        }
    }

    fn readable(&mut self,
                event_loop: &mut EventLoop,
                token: Token)
                -> io::Result<()> {
        let messages = try!(self.connections[token].readable());

        for message in messages {
            let post = match Post::from_bytes(message) {
                Err(e) => {
                    error!("Invalid message: {}", e);
                    continue;
                },
                Ok(post) => post
            };
            if post.text.len() > 0 && post.text.starts_with("/") {
                self.shell.send(Task {
                    user: post.author,
                    cmd: post.text[1..].to_string(),
                    reply_to: event_loop.channel(),
                }).unwrap_or_else(|e| error!("failed to execute command {}", e))
            } else {
                self.broadcast(event_loop, post);
            }
        }
        Ok(())
    }

    fn broadcast(&mut self, event_loop: &mut EventLoop, post: Post) {
        for p in self.peers.iter() {
            if let Err(e) = p.send(Message::NewPost(post.clone())) {
                error!("cannot forward post to peer, {:?}", e);
            }
        }
        let bytes = post.into_bytes();
        self.broadcast_local(event_loop, &bytes)
    }

    fn broadcast_local(&mut self, event_loop: &mut EventLoop, message: &[u8]) {
        let mut bad_tokens = Vec::new();
        for conn in self.connections.iter_mut() {
            let buf = ByteBuf::from_slice(&message);
            conn.send_message(buf)
                .and_then(|_| conn.reregister(event_loop))
                .unwrap_or_else(|e| {
                    error!("Failed to send message for {:?}: {:?}", conn.token, e);
                    bad_tokens.push(conn.token);
                });
        }

        for t in bad_tokens {
            self.reset_connection(t);
        }
    }



    fn reset_connection(&mut self, token: Token) {
        info!("reset connection {:?}", token);
        self.connections.remove(token);
    }
}


impl mio::Handler for Worker {
    type Timeout = ();
    type Message = Message;

    fn ready(&mut self,
             event_loop: &mut EventLoop,
             token: Token,
             events: EventSet) {

        debug!("events = {:?}", events);
        assert!(token != Token(0), "[BUG]: Received event for Token(0)");

        if events.is_error() {
            warn!("Error event for {:?}", token);
            self.reset_connection(token);
            return;
        }

        if events.is_hup() {
            trace!("Hup event for {:?}", token);
            self.reset_connection(token);
            return;
        }

        if events.is_writable() {
            trace!("Write event for {:?}", token);
            assert!(self.token != token, "Received writable event for server");

            self.connections[token].writable()
                .and_then(|_| self.connections[token].reregister(event_loop))
                .unwrap_or_else(|e| {
                    error!("Write event failed for {:?}, {:?}", token, e);
                    self.reset_connection(token);
                });
        }

        if events.is_readable() {
            trace!("Read event for {:?}", token);
            self.readable(event_loop, token)
                .and_then(|_| self.connections[token].reregister(event_loop))
                .unwrap_or_else(|e| {
                    error!("Read event failed for {:?}: {:?}", token, e);
                    self.reset_connection(token);
                });
        }

    }

    fn notify(&mut self, event_loop: &mut EventLoop, msg: Message) {
        info!("Worker {} received a message {:?}", self.id, msg);
        match msg {
            Message::NewConnection(sock) => {
                self.accept(event_loop, sock)
            }
            Message::TaskFinished { user, result } => {
                let post = Post {
                    author: user,
                    text: result
                };
                self.broadcast(event_loop, post)
            }
            Message::NewPost(post) => self.broadcast_local(event_loop, &post.into_bytes())
        }
    }
}

