use maelstrom_rust::{Message, Payload, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Payload)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Payload)]
pub struct InitOk {}

fn handle_init(state: Arc<State>, msg: Message<Init>) -> InitOk {
    let init = msg.body.payload;
    let mut inner = state.inner.lock().unwrap();
    inner.node_id = Some(init.node_id);
    inner.node_ids = Some(init.node_ids);
    InitOk {}
}

#[derive(Serialize, Deserialize, Payload)]
struct Echo {
    echo: String,
}

#[derive(Serialize, Deserialize, Payload)]
struct EchoOk {
    echo: String,
}

fn handle_echo(_: Arc<State>, msg: Message<Echo>) -> EchoOk {
    let echo = msg.body.payload;
    EchoOk { echo: echo.echo }
}

struct State {
    inner: Mutex<Inner>,
}

impl State {
    fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new()),
        }
    }
}

struct Inner {
    node_id: Option<String>,
    node_ids: Option<Vec<String>>,
}

impl Inner {
    fn new() -> Self {
        Self {
            node_id: None,
            node_ids: None,
        }
    }
}

fn main() {
    let state = Arc::new(State::new());
    let mut reg = Router::new(state);
    reg.register(handle_echo);
    reg.register(handle_init);
    reg.serve();
}
