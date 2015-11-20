use std::thread;
use std::sync::mpsc;
use std::process::{Command, Output};

use mio;

use super::Message;

pub struct Task {
    pub user: String,
    pub cmd: String,
    pub reply_to: mio::Sender<Message>
}

pub fn start() -> mpsc::Sender<Task> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
       loop {
           match rx.recv() {
               Err(e) => error!("Failed to receive shell message: {:?}", e),
               Ok(Task {user, cmd, reply_to} ) => {
                   let msg = Message::TaskFinished {
                       user: user,
                       result: exec(&cmd)
                   };
                   if let Err(e) = reply_to.send(msg) {
                       error!("Failed to deliver shell message: {:?}", e)
                   }
               }
           }
       }
    });
    return tx;
}

pub fn exec(cmd: &str) -> Vec<String> {
    let args = cmd.split_whitespace().collect::<Vec<_>>();
    if args.len() == 0 {
        return vec!["Bad command".to_string()];
    }

    let text = match Command::new(args[0]).args(&args[1..]).output() {
        Err(e) => format!("Command {} failed: {}", cmd, e),
        Ok(Output {stdout, .. }) => String::from_utf8_lossy(&stdout).to_string()
    };

    text.lines().map(|s| s.to_string()).collect::<Vec<_>>()
}
