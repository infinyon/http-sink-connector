# Contributing to Infinyon's HTTP Sink Connector

This repository scaffold follows Rust's Workspaces structure, theres two crates
contained for this project:

1. `http-sink` which stores Connector source code
2. `tiny-http-server`, a HTTP server implementation used for testing

## Run and test HTTP sink code

1. Build and run mockup `tiny-http-server` in this package

```
cargo run --bin tiny-http-server
```

The mockup server runs at `127.0.0.1:8080` and it echos back requests.

2. Build and Run http-sink via `cdk` (assuming you have cdk installed)

> You can install CDK using `fluvio install cdk`

```bash
cd ./crates/http-sink
cdk build
cdk test -c config-example.yaml
```

3. Produce on `http-sink` topic

```
fluvio produce http-sink
> test
Ok!
> {"request": "hello world"}
Ok!
```

The mock-up server will print the request:

```
Request: Post, url: "/", headers: [Header { field: HeaderField("user-agent"), value: "fluvio/http-sink 0.1.0" }, Header { field: HeaderField("accept"), value: "*/*" }, Header { field: HeaderField("host"), value: "127.0.0.1:8080" }, Header { field: HeaderField("content-length"), value: "26" }]
Content: "{\"request\": \"hello world\"}"
```
