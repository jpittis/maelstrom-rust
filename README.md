The beginnings of a typesafe [maelstrom](https://github.com/jepsen-io/maelstrom) framework for Rust.

The existing echo example has passed the maelstrom echo test:

```
maelstrom test -w echo --bin target/debug/examples/echo --node-count 1 --time-limit 10
```

Things I haven't supported yet that I plan to:

- [ ] Getting rid of the random unwraps in the message parsing code.
- [ ] First-class async support.
- [ ] A typesafe RPC client.
