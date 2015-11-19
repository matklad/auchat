use std::io::{self, Error, ErrorKind};

use mio::{self, Token, EventSet, PollOpt, TryRead, TryWrite};
use mio::tcp::*;
use mio::buf::{ByteBuf, Buf};

use super::chunker::Chunker;
use super::worker::Worker;

type EventLoop = mio::EventLoop<Worker>;


pub struct Connection {
    pub token: mio::Token,
    socket: TcpStream,
    interest: EventSet,
    send_queue: Vec<ByteBuf>,
    chunker: Chunker,
}

impl Connection {
    pub fn new(socket: TcpStream, token: mio::Token) -> Connection {
        Connection {
            socket: socket,
            token: token,
            interest: EventSet::hup(),
            send_queue: Vec::new(),
            chunker: Chunker::new(),
        }
    }

    pub fn register(&mut self, event_loop: &mut EventLoop) -> io::Result<()> {
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

    pub fn reregister(&mut self, event_loop: &mut EventLoop) -> io::Result<()> {
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
                    if n < recv_buf.capacity() {
                        break;
                    }
                },
                Err(e) => return Err(e)
            }
        }

        Ok(self.chunker.feed(recv_buf.flip().bytes()))
    }

    pub fn writable(&mut self) -> io::Result<()> {
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
