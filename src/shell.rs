use std::thread;
use std::sync::mpsc;
use std::process::{Command, Output};

use mio;

use super::Message;

pub struct Task {
    pub cmd: String,
    pub reply_to: mio::Sender<Message>
}

pub fn start() -> mpsc::Sender<Task> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
       loop {
           match rx.recv() {
               Err(e) => error!("Failed to receive shell message: {:?}", e),
               Ok(Task {cmd, reply_to} ) => {
                   let msg = Message::TaskFinished(exec(&cmd));
                   if let Err(e) = reply_to.send(msg) {
                       error!("Failed to deliver shell message: {:?}", e)
                   }
               }
           }
       }
    });
    return tx;
}

pub fn exec(cmd: &str) -> String {
    match Command::new(cmd).output() {
        Err(e) => format!("Command {} failed: {}", cmd, e),
        Ok(Output {stdout, .. }) => String::from_utf8_lossy(&stdout).to_string()
    }
}
