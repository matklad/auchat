use std::io::{self, Write};
use std::net::SocketAddr;
use std::marker::PhantomData;
use std::thread;

use mio::{self, Token, EventSet, PollOpt, TryRead, TryWrite};
use mio::buf::{Buf, ByteBuf};
use mio::tcp::{TcpStream, TcpListener};
use mio::util::Slab;

use super::chunker::Chunker;
use super::ProtoHandler;
use super::utils::*;

enum WorkerMessage<H: ProtoHandler> {
    NewConnection(TcpStream),
    HandlerMessage(Token, H::Message),
    Broadcast(H::Proto),
}

type Workers<H: ProtoHandler> = Vec<mio::Sender<WorkerMessage<H>>>;
//
pub struct ProtoServer<H: ProtoHandler> {
    socket: TcpListener,
    token: Token,
    workers: Workers<H>,
    worker_ptr: usize,
}

pub struct User<H: ProtoHandler> {
    sender: Sender<H>,
    broadcast: Option<H::Proto>,
    echo: Option<H::Proto>,
}

impl<H: ProtoHandler> User<H> {
    fn new(token: Token, sender: mio::Sender<WorkerMessage<H>>) -> Self {
        User {
            sender: Sender { token: token, sender: sender},
            broadcast: None,
            echo: None,
        }
    }

    pub fn channel(&self) -> Sender<H> {
        self.sender.clone()
    }

    pub fn broadcast(&mut self, message: H::Proto) {
        self.broadcast = Some(message);
    }

    pub fn echo(&mut self, message: H::Proto) {
        self.echo = Some(message);
    }
}

impl<H: ProtoHandler> ProtoServer<H> {
    pub fn start(addr: SocketAddr, handler: H, n_workers: usize) {
        let socket = TcpListener::bind(&addr)
        .ok().expect("Failed to bind address");

        let mut event_loop = mio::EventLoop::<Self>::new()
        .ok().expect("Failed to create event loop");

        let workers = Worker::start(handler, n_workers);
        let mut server = ProtoServer {
            socket: socket,
            token: mio::Token(1),
            workers: workers,
            worker_ptr: 0,
        };
        server.register(&mut event_loop)
              .ok().expect("Failed to register server with event loop");

        event_loop.run(&mut server)
                  .ok().expect("Failed to start event loop");
    }
    fn accept(&mut self, event_loop: &mut mio::EventLoop<Self>) {
        info!("accepted a new client socket");
        let socket = match self.socket.accept() {
            Ok(Some(sock)) => sock,
            Ok(None) => {
                error!("Failed to accept a new socket");
                self.reregister(event_loop);
                return;
            }
            Err(e) => {
                error!("Error while accepting connection, {:?}", e);
                self.reregister(event_loop);
                return;
            }
        };

        if let Err(e) = self.round_robin(socket) {
            error!("Failed to deliver connection to client, {:?}", e);
        }

        self.reregister(event_loop);
    }

    fn round_robin(&mut self, sock: TcpStream)
    -> Result<(), mio::NotifyError<WorkerMessage<H>>> {
        self.worker_ptr += 1;
        self.worker_ptr %= self.workers.len();

        let msg = WorkerMessage::NewConnection(sock);
        self.workers[self.worker_ptr].send(msg)
    }

    pub fn register(&mut self, event_loop: &mut mio::EventLoop<Self>)
    -> io::Result<()> {
        event_loop.register_opt(
            &self.socket,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to register server {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    fn reregister(&mut self, event_loop: &mut mio::EventLoop<Self>) {
        event_loop.reregister(
            &self.socket,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).unwrap_or_else(|e| {
            error!("Failed to reregister server {:?}, {:?}", self.token, e);
            event_loop.shutdown();
        })
    }

}

#[derive(Clone)]
pub struct Sender<H: ProtoHandler> {
    token: Token,
    sender: mio::Sender<WorkerMessage<H>>,
}

impl<H: ProtoHandler> Sender<H> {
    pub fn send(&self, message: H::Message)
                -> Result<(), mio::NotifyError<WorkerMessage<H>>> {
        self.sender.send(WorkerMessage::HandlerMessage(self.token, message))
    }
}

impl<H: ProtoHandler> mio::Handler for ProtoServer<H> {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self,
    event_loop: &mut mio::EventLoop<Self>,
    token: mio::Token,
    events: mio::EventSet) {
        debug!("events = {:?}", events);
        assert!(token != mio::Token(0), "[BUG]: Received event for Token(0)");
        assert!(self.token == token, "Received writable event for server");

        if events.is_error() {
            error!("Error event for server");
            event_loop.shutdown();
            return;
        }

        if events.is_readable() {
            trace!("Read event for {:?}", token);
            assert!(token == self.token, "Server received unexpected token");
            self.accept(event_loop);
        }
    }
}

struct Worker<H: ProtoHandler> {
    id: usize,
    handler: H,
    connections: Slab<Connection<H>>,
    peers: Workers<H>,
}

impl<H: ProtoHandler> Worker<H> {

    fn new(id: usize, handler: H, peers: Workers<H>) -> Self {
        assert!(id > 0);
        Worker {
            id: id,
            handler: handler,
            connections: Slab::new_starting_at(Token(100 * id), 100_000),
            peers: peers,
        }
    }

    pub fn start(handler: H, n_workers: usize) -> Workers<H> {
        assert!(n_workers > 0, "Need at least one worker");
        let loops = (0..n_workers)
        .map(|_| mio::EventLoop::<Self>::new().ok().expect("Failed to crate a worker"))
        .collect::<Vec<_>>();
        let chans = loops.iter().map(|l| l.channel()).collect::<Vec<_>>();

        for (id, mut l) in loops.into_iter().enumerate() {
            let peers = (0..n_workers)
            .filter(|&i| i != id).map(|i| chans[i].clone()).collect();
            let handler = handler.clone();

            thread::spawn(move || {
                l.run(&mut Worker::new(id + 1, handler, peers))
                .ok().expect("Failed to start a worker event loop");
            });

        }

        chans
    }

    fn accept(&mut self, event_loop: &mut mio::EventLoop<Self>, sock: TcpStream) {
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
                event_loop: &mut mio::EventLoop<Self>,
                token: Token)
                -> io::Result<()> {

        let messages = try!(self.connections[token].readable());
        for message in messages {
            let proto = match from_bytes::<H::Proto>(&message) {
                Err(e) => {
                    error!("Invalid message: {}", e);
                    continue;
                }
                Ok(proto) => proto
            };

            let mut user = User::new(token, event_loop.channel());
            self.handler.recv(&mut user, proto);

            self.perform_requests(event_loop, token, user);
        }

        Ok(())
    }

    fn perform_requests(&mut self,
                        event_loop: &mut mio::EventLoop<Self>,
                        token: Token,
                        user: User<H>) {
        let  User {broadcast, echo, ..} = user;

        if let Some(proto) = broadcast {
            self.broadcast(event_loop, proto);
        }

        if let Some(proto) = echo {
            let buf = ByteBuf::from_slice(&to_bytes(&proto));
            self.connections[token].send_message(buf)
                .and_then(|_| self.connections[token].reregister(event_loop))
                .unwrap_or_else(|_| self.reset_connection(token));
        }
    }

    fn reset_connection(&mut self, token: Token) {
        info!("reset connection {:?}", token);
        self.connections.remove(token);
    }

    fn broadcast(&mut self, event_loop: &mut mio::EventLoop<Self>, proto: H::Proto) {
        for p in self.peers.iter() {
            if let Err(e) = p.send(WorkerMessage::Broadcast(proto.clone())) {
                error!("cannot forward post to peer, {:?}", e);
            }
        }
        self.broadcast_local(event_loop, proto)
    }

    fn broadcast_local(&mut self, event_loop: &mut mio::EventLoop<Self>, proto: H::Proto) {
        let message = to_bytes(&proto);
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
}


impl<H: ProtoHandler> mio::Handler for Worker<H> {
    type Timeout = ();
    type Message = WorkerMessage<H>;

    fn ready(&mut self,
    event_loop: &mut mio::EventLoop<Self>,
    token: Token,
    events: EventSet) {

        debug!("events = {:?}", events);
        assert!(token != Token(0), "[BUG]: Received event for Token(0)");

        if events.is_error() || events.is_hup() {
            if events.is_error() {
                error!("Error event for {:?}", token);
            } else {
                warn!("Hup event for {:?}", token);
            }

            self.reset_connection(token);
            return;
        }

        if events.is_writable() {
            trace!("Write event for {:?}", token);

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

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Self>, msg: Self::Message) {
        info!("Worker {} received a message", self.id);
        match msg {
            WorkerMessage::NewConnection(sock) => {
                self.accept(event_loop, sock)
            }
            WorkerMessage::HandlerMessage(token, m) => {
                let mut user = User::new(token, event_loop.channel());
                self.handler.notify(&mut user, m);
                self.perform_requests(event_loop, token, user);
            }
            WorkerMessage::Broadcast(post) => {
                self.broadcast_local(event_loop, post)
            }
        }
    }

}

struct Connection<H: ProtoHandler> {
    token: mio::Token,
    socket: TcpStream,
    interest: EventSet,
    send_queue: Vec<ByteBuf>,
    chunker: Chunker,
    h: PhantomData<H>,
}

impl<H: ProtoHandler> Connection<H> {
    pub fn new(socket: TcpStream, token: Token) -> Self {
        Connection {
            socket: socket,
            token: token,
            interest: EventSet::hup(),
            send_queue: Vec::new(),
            chunker: Chunker::new(),
            h: PhantomData,
        }
    }

    pub fn register(&mut self, event_loop: &mut mio::EventLoop<Worker<H>>) -> io::Result<()> {
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

    pub fn reregister(&mut self, event_loop: &mut mio::EventLoop<Worker<H>>) -> io::Result<()> {
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

    pub fn send_message(&mut self, message: ByteBuf) -> io::Result<()> {
        self.send_queue.push(message);
        self.interest.insert(EventSet::writable());
        Ok(())
    }

    pub fn readable(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut recv_buf = ByteBuf::mut_with_capacity(2048);

        loop {
            match self.socket.try_read_buf(&mut recv_buf) {
                Ok(None) => break,
                Ok(Some(n)) => {
                    debug!("Read {} bytes for {:?}", n, self.token);
                    if n < recv_buf.capacity() {
                        break;
                    }
                },
                Err(e) => return Err(e)
            }
        }

        Ok(self.chunker.feed(recv_buf.flip()))
    }

    pub fn writable(&mut self) -> io::Result<()> {
        debug!("queue size for {:?} is {}", self.token, self.send_queue.len());

        while let Some(mut buf) = self.send_queue.pop() {
            match self.socket.try_write_buf(&mut buf) {
                Ok(None) => {
                    debug!("client flushing buf");
                    self.send_queue.push(buf);
                    break;
                }
                Ok(Some(n)) => {
                    debug!("Wrote {} bytes for {:?}", n, self.token);
                    if buf.has_remaining() {
                        self.send_queue.push(buf);
                        break;
                    }
                },
                Err(e) => {
                    error!("Failed to send buffer for {:?}, error: {:?}", self.token, e);
                    return Err(e);
                }
            }
        }

        if self.send_queue.is_empty() {
            self.interest.remove(EventSet::writable());
        }

        Ok(())
    }
}
