use std::io;

use mio::{self, Token, EventSet, PollOpt, TryRead, TryWrite};
use mio::tcp::*;
use mio::buf::{Buf, ByteBuf};

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
