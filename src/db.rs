//! Do not change anything inside the `db` module

use std::collections::VecDeque;

/// Stores some data as a queue
pub struct Database {
  store: VecDeque<String>,
}

/// Commands supported by the database
#[derive(Eq, PartialEq, Debug)]
pub enum Command {
  Pub(String),
  Get,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
  UnnownCmd,
  BadPayload,
  Incomplete,
}

pub fn parse(input: &str) -> Result<Command, Error> {
  let mut split = input.splitn(2, ' ');

  if let Some(verb) = split.next() {
    match verb.trim() {
      "GET" => {
        if split.next() == None {
          Ok(Command::Get)
        } else {
          Err(Error::BadPayload)
        }
      }
      "PUB" => {
        if let Some(payload) = split.next() {
          Ok(Command::Pub(payload.trim().into()))
        } else {
          Err(Error::BadPayload)
        }
      }
      "" => Err(Error::Incomplete),
      _ => Err(Error::UnnownCmd),
    }
  } else {
    Err(Error::Incomplete)
  }
}

impl Database {
  pub fn new() -> Self {
    Self {
      store: VecDeque::new(),
    }
  }

  pub fn get(&mut self) -> Option<String> {
    self.store.pop_back()
  }

  pub fn store(&mut self, msg: String) {
    self.store.push_back(msg);
  }
}
