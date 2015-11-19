use std::io::{self, Error, ErrorKind};
use std::thread;
use std::sync::mpsc;

use mio::{self, Token, EventSet, PollOpt, TryRead, TryWrite};
use mio::tcp::*;
use mio::buf::{ByteBuf, Buf};
use mio::util::Slab;

use Message;
use shell::Task;
use chunker::Chunker;
use post::Post;


pub struct Worker {
    id: usize,
    token: Token,
    connections: Slab<Connection>,
    shell: mpsc::Sender<Task>,
}

type EventLoop = mio::EventLoop<Worker>;


impl Worker {
    fn new(id: usize, shell: mpsc::Sender<Task>) -> Worker {
        assert!(id > 0);
        let worker_token = 100 * id - 1;
        Worker {
            id: id,
            token: Token(worker_token),
            connections: Slab::new_starting_at(Token(worker_token + 1), 1024),
            shell: shell,
        }
    }

    pub fn start(n_workers: usize, shell: &mpsc::Sender<Task>)
                 -> Vec<mio::Sender<Message>> {
        assert!(n_workers > 0, "Need at least one worker");
        let mut result = Vec::new();

        for id in 0..n_workers {
            let mut event_loop = EventLoop::new()
            .ok().expect("Failed to crate a worker");
            result.push(event_loop.channel());
            let shell = shell.clone();
            thread::spawn(move || {
                let mut w = Worker::new(id + 1, shell);
                event_loop.run(&mut w)
                .ok().expect("Failed to start a worker event loop");
            });
        }

        result
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

            let resp = post.into_bytes();
            self.broadcast(event_loop, &resp);
        }
        Ok(())
    }

    fn broadcast(&mut self, event_loop: &mut EventLoop, message: &[u8]) {
        let mut bad_tokens = Vec::new();
        for conn in self.connections.iter_mut() {
            let buf = ByteBuf::from_slice(&message);
            conn.send_message(buf)
                .and_then(|_| conn.reregister(event_loop))
                .unwrap_or_else(|e| {
                    error!("Failed to echo message for {:?}: {:?}", conn.token, e);
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
            Message::TaskFinished(result) => {
                self.broadcast(event_loop, &result.into_bytes())
            }
        }
    }
}

struct Connection {
    socket: TcpStream,
    token: mio::Token,
    interest: EventSet,
    send_queue: Vec<ByteBuf>,
    chunker: Chunker,
}

impl Connection {
    fn new(socket: TcpStream, token: mio::Token) -> Connection {
        Connection {
            socket: socket,
            token: token,
            interest: EventSet::hup(),
            send_queue: Vec::new(),
            chunker: Chunker::new(),
        }
    }

    fn register(&mut self, event_loop: &mut EventLoop) -> io::Result<()> {
        self.interest.insert(EventSet::readable());

        event_loop.register_opt(
            &self.socket,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to register {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    fn reregister(&mut self, event_loop: &mut EventLoop) -> io::Result<()> {
        event_loop.reregister(
            &self.socket,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }


    fn send_message(&mut self, message: ByteBuf) -> io::Result<()> {
        self.send_queue.push(message);
        self.interest.insert(EventSet::writable());
        Ok(())
    }

    fn readable(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut recv_buf = ByteBuf::mut_with_capacity(2048);

        loop {
            match self.socket.try_read_buf(&mut recv_buf) {
                Ok(None) => break,
                Ok(Some(n)) => {
                    if n < recv_buf.capacity() {
                        break;
                    }
                },
                Err(e) => return Err(e)
            }
        }

        Ok(self.chunker.feed(recv_buf.flip().bytes()))
    }

    fn writable(&mut self) -> io::Result<()> {
        try!(self.send_queue.pop()
             .ok_or(Error::new(ErrorKind::Other, "Could not pop send queue"))
             .and_then(|mut buf| {
                 match self.socket.try_write_buf(&mut buf) {
                     Ok(None) => {
                         debug!("client flushing buf");
                         self.send_queue.push(buf);
                         Ok(())
                     }
                     Ok(Some(n)) => {
                         debug!("Wrote {} bytes for {:?}", n, self.token);
                         Ok(())
                     },
                     Err(e) => {
                         error!("Failed to send buffer for {:?}, error: {:?}",
                                self.token, e);
                         Err(e)
                     }
                 }
             })
        );
        if self.send_queue.is_empty() {
            self.interest.remove(EventSet::writable());
        }

        Ok(())
    }
}

