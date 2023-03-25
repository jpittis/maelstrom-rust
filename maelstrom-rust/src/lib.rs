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

impl<T> Message<T> {
    fn from_raw(raw: RawMessage, payload: T) -> Message<T> {
        Message {
            src: raw.src,
            dest: raw.dest,
            body: Body {
                msg_id: raw.body.msg_id,
                in_reply_to: raw.body.in_reply_to,
                payload,
            },
        }
    }
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

type WrappedHandler<S> = Box<dyn FnMut(S, RawMessage) -> RawMessage>;

pub struct Router<S = ()> {
    store: HashMap<&'static str, WrappedHandler<S>>,
    state: S,
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(state: S) -> Self {
        Self {
            store: HashMap::new(),
            state,
        }
    }

    pub fn register<F, T, Q>(&mut self, mut handler: F)
    where
        F: FnMut(S, Message<T>) -> Q + 'static,
        T: Payload,
        Q: Payload,
    {
        let ttag = T::tag();
        let qtag = Q::tag();
        let wrapper = Box::new(move |state: S, raw_req: RawMessage| {
            let mut raw_rep = raw_req.clone();
            let payload_req =
                serde_json::from_value(serde_json::Value::Object(raw_req.body.payload.clone()))
                    .unwrap();
            let payload_rep = handler(state, Message::from_raw(raw_req, payload_req));
            let raw_payload_rep = to_json_map(payload_rep);
            raw_rep.body.payload = raw_payload_rep;
            raw_rep.body.r#type = qtag.to_string();
            raw_rep.body.in_reply_to = raw_rep.body.msg_id;
            raw_rep.body.msg_id = None;
            std::mem::swap(&mut raw_rep.src, &mut raw_rep.dest);
            raw_rep
        });
        self.store.insert(ttag, wrapper);
    }

    fn send(&mut self, raw_req: RawMessage) -> RawMessage {
        let tag = &raw_req.body.r#type as &str;
        let wrapper = self.store.get_mut(tag).unwrap();
        let raw_rep = wrapper.as_mut()(self.state.clone(), raw_req);
        raw_rep
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

    fn handle_echo(_: (), msg: Message<Echo>) -> EchoOk {
        let echo = msg.body.payload;
        EchoOk { text: echo.text }
    }

    #[test]
    fn test_derive_payload_tagging() {
        assert_eq!("echo", Echo::tag());
        assert_eq!("echo_ok", EchoOk::tag());
    }

    #[test]
    fn test_register() {
        let mut reg = Router::new(());
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
