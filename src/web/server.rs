use std::io;

use mio::{self, Token, EventSet, PollOpt};
use mio::tcp::*;

use Message;


pub struct Server {
    socket: TcpListener,
    token: Token,
    workers: Vec<mio::Sender<Message>>,
    worker_ptr: usize,
}

type EventLoop = mio::EventLoop<Server>;

impl Server {
    pub fn new(socket: TcpListener, workers: Vec<mio::Sender<Message>>) -> Server {
        Server {
            socket: socket,
            token: Token(1),
            workers: workers,
            worker_ptr: 0,
        }
    }

    pub fn register(&mut self, event_loop: &mut EventLoop) -> io::Result<()> {
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

    fn reregister(&mut self, event_loop: &mut EventLoop) {
        event_loop.reregister(
            &self.socket,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).unwrap_or_else(|e| {
            error!("Failed to reregister server {:?}, {:?}", self.token, e);
            self.shutdown(event_loop);
        })
    }

    fn shutdown(&mut self, event_loop: &mut EventLoop) {
        event_loop.shutdown();
    }

    fn send_to_worker(&mut self, msg: Message)
                      -> Result<(), mio::NotifyError<Message>> {
        let result = self.workers[self.worker_ptr].send(msg);
        self.worker_ptr += 1;
        self.worker_ptr %= self.workers.len();
        result
    }

    fn accept(&mut self, event_loop: &mut EventLoop) {
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

        if let Err(e) = self.send_to_worker(Message::NewConnection(socket)) {
            error!("Failed to deliver connection to client, {:?}", e);
        }

        self.reregister(event_loop);
    }
}

impl mio::Handler for Server {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self,
             event_loop: &mut EventLoop,
             token: Token,
             events: EventSet) {
        debug!("events = {:?}", events);
        assert!(token != Token(0), "[BUG]: Received event for Token(0)");
        assert!(self.token == token, "Received writable event for server");

        if events.is_error() {
            error!("Error event for server");
            self.shutdown(event_loop);
            return;
        }

        if events.is_readable() {
            trace!("Read event for {:?}", token);
            assert!(token == self.token, "Server received unexpected token");
            self.accept(event_loop);
        }
    }
}


