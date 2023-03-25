pub use maelstrom_rust_derive::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RawMessage {
    pub src: String,
    pub dest: String,
    pub body: RawBody,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RawBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<usize>,
    pub r#type: String,
    #[serde(flatten)]
    pub payload: serde_json::map::Map<String, serde_json::Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message<T> {
    pub src: String,
    pub dest: String,
    pub body: Body<T>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Body<T> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    pub payload: T,
}

pub trait Payload: Sized + Serialize + DeserializeOwned {
    fn tag() -> &'static str;
}

pub struct Router {
    store: HashMap<&'static str, Box<dyn FnMut(RawMessage) -> RawMessage>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn register<F, T, Q>(&mut self, mut handler: F)
    where
        F: FnMut(T) -> Q + 'static,
        T: Payload,
        Q: Payload,
    {
        let tag = T::tag();
        let wrapper = Box::new(move |raw_req: RawMessage| {
            let mut raw_rep = raw_req.clone();
            let payload_req =
                serde_json::from_value(serde_json::Value::Object(raw_req.body.payload)).unwrap();
            let payload_rep = handler(payload_req);
            let raw_payload_rep = to_json_map(payload_rep);
            raw_rep.body.payload = raw_payload_rep;
            raw_rep
        });
        self.store.insert(tag, wrapper);
    }

    fn send(&mut self, raw: RawMessage) -> RawMessage {
        let tag = &raw.body.r#type as &str;
        let wrapper = self.store.get_mut(tag).unwrap();
        wrapper.as_mut()(raw)
    }

    pub fn serve(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        for line in stdin.lock().lines() {
            let raw_req: RawMessage = serde_json::from_str(&line.unwrap()).unwrap();
            let raw_rep = self.send(raw_req);
            stdout
                .write_all(&serde_json::to_vec(&raw_rep).unwrap())
                .unwrap();
            stdout.write_all(b"\n").unwrap();
            stdout.flush().unwrap();
        }
    }
}

fn to_json_map<T: Payload>(payload: T) -> serde_json::Map<String, serde_json::Value> {
    let raw_payload = serde_json::to_value(payload).unwrap();
    match raw_payload {
        serde_json::Value::Object(obj) => obj,
        _ => panic!("Expected payload to serialize to JSON map"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Serialize, Deserialize, Payload)]
    struct Echo {
        text: String,
    }

    #[derive(Serialize, Deserialize, Payload)]
    struct EchoOk {
        text: String,
    }

    fn handle_echo(echo: Echo) -> EchoOk {
        EchoOk { text: echo.text }
    }

    #[test]
    fn test_derive_payload_tagging() {
        assert_eq!("echo", Echo::tag());
        assert_eq!("echo_ok", EchoOk::tag());
    }

    #[test]
    fn test_register() {
        let mut reg = Router::new();
        reg.register(handle_echo);
        let raw_payload = serde_json::Map::from_iter(vec![(
            "text".to_string(),
            serde_json::Value::String("hello".to_string()),
        )]);
        let raw_req = RawMessage {
            src: "node1".to_string(),
            dest: "node2".to_string(),
            body: RawBody {
                msg_id: Some(1),
                in_reply_to: None,
                r#type: "echo".to_string(),
                payload: raw_payload.clone(),
            },
        };
        let raw_rep = reg.send(raw_req);
        assert_eq!(raw_rep.body.payload, raw_payload)
    }
}
