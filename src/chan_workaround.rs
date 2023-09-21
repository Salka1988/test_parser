use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json;
use std::io::Read;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::{fmt, thread};

type DeserializeResult = Result<MyJson, String>;

#[derive(Deserialize, Debug)]
pub struct MyJson {
    val1: String,
    val2: Vec<i32>,
}

pub struct MyJsonIterator {
    receiver: Receiver<DeserializeResult>,
}

pub struct MyJsonVisitor {
    sender: SyncSender<DeserializeResult>,
}

impl Iterator for MyJsonIterator {
    type Item = DeserializeResult;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv().ok() //ok() because a RecvError implies we are done
    }
}

impl MyJsonIterator {
    pub fn new(reader: impl Read + Send + 'static) -> Self {
        let (sender, receiver) = sync_channel::<DeserializeResult>(0);

        thread::spawn(move || {
            let mut deserializer = serde_json::Deserializer::from_reader(reader);
            if let Err(e) = deserializer.deserialize_seq(MyJsonVisitor {
                sender: sender.clone(),
            }) {
                let _ = sender.send(Err(e.to_string())); //let _ = because error from calling send just means receiver has disconnected
            }
        });

        Self { receiver }
    }
}

impl<'de> Visitor<'de> for MyJsonVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("array of MyJson")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
    {
        while let Some(val) = seq.next_element::<MyJson>()? {
            if self.sender.send(Ok(val)).is_err() {
                break; //receiver has disconnected.
            }
        }
        Ok(())
    }
}
