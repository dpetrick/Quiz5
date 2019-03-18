mod db;
mod tp;

use db::{Command, Database};
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use tp::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Store data
    let storage = Arc::new(Mutex::new(Database::new()));

    // Creates a threadpool with 4 connection workers
    let mut pool = ThreadPool::new(4);

    // This is an infinite iterator
    for stream in listener.incoming() {
        let storage = Arc::clone(&storage);
        let mut stream = stream.unwrap();
        let mut read_buffer = String::new();
        let mut buffered_stream = BufReader::new(&stream);
        if let Err(_) = buffered_stream.read_line(&mut read_buffer) {
            break;
        }

        let cmd = db::parse(&read_buffer);
        pool.queue(move || {
            let mut storage = storage.lock().expect("Tainted mutex! Oh no!");
            match cmd {
                Ok(Command::Get) => send_reply(
                    &mut stream,
                    storage.get().unwrap_or_else(|| "<empty>".into()),
                ),
                Ok(Command::Pub(s)) => {
                    storage.store(s);
                    send_reply(&mut stream, "<done>");
                }
                Err(e) => send_reply(&mut stream, format!("<error: {:?}>", e)),
            }
        });
    }

    // Join threads & cleanup
    pool.shutdown()
}

// No need to really touch this function
fn send_reply<'a, S: Into<String>>(stream: &mut TcpStream, msg: S) {
    // Sometimes we break and we don't care
    let _ = stream.write(msg.into().as_bytes());
    let _ = stream.write("\r\n".as_bytes());
}
