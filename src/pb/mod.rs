use std::net::SocketAddr;


use protobuf;

mod server;
mod utils;
mod chunker;

pub use self::server::Sender;
pub use self::server::User;

pub fn start_server<H: ProtoHandler>(addr: SocketAddr, handler: H, n_workers: usize) {
    server::ProtoServer::start(addr, handler, n_workers);
}

pub trait ProtoHandler: Sized + Clone + Send + 'static {
    type Proto: protobuf::Message + protobuf::MessageStatic + Send;
    type Message: Send + Clone;

    fn recv(&mut self, user: &mut User<Self>, message: Self::Proto);
    fn notify(&mut self, user: &mut User<Self>, message: Self::Message);
}
