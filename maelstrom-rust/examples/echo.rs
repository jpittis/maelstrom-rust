use maelstrom_rust::{Payload, Router};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Payload)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Payload)]
pub struct InitOk {}

fn handle_init(init: Init) -> InitOk {
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

fn handle_echo(echo: Echo) -> EchoOk {
    EchoOk { echo: echo.echo }
}

fn main() {
    let mut reg = Router::new();
    reg.register(handle_echo);
    reg.register(handle_init);
    reg.serve();
}
