# ai-at-home
AI Edge Computing

A reference implementation of **AI@home** – an AI edge computing infrastructure consisting of:

1. **`broker`** – a registry service where AI nodes advertise their availability and AI clients discover them.
2. **`node`** – a basic AI node server that registers with the broker, accepts inference requests, and returns responses.
3. **`common`** – shared data-model types used by both services.

---

## Architecture

```
                ┌──────────────────────────────────────────────┐
                │                   Broker                     │
                │  POST   /api/v1/nodes  – node registers      │
                │  DELETE /api/v1/nodes/:id – node leaves      │
                │  GET    /api/v1/nodes  – clients discover    │
                │  GET    /health        – health check        │
                └──────────────────────────────────────────────┘
                       ▲  register/deregister       ▲ discover
                       │                            │
               ┌───────┴────────┐         ┌────────┴────────┐
               │   AI Node(s)   │         │   AI Client(s)  │
               │                │         │                 │
               │  POST /api/v1/infer      │  (any HTTP      │
               │  GET  /health  │         │   client)       │
               └────────────────┘         └─────────────────┘
```

---

## Quick start

### Prerequisites

* [Rust](https://rustup.rs/) 1.75 or later

### Build everything

```bash
cargo build
```

### Run tests

```bash
cargo test
```

### Start the broker

```bash
cargo run --bin broker
# Broker listening on 0.0.0.0:3000
```

### Start a node

In a separate terminal:

```bash
cargo run --bin node
# Node 'ai-node' listening on 0.0.0.0:4000
```

The node automatically registers with the broker on start-up and deregisters on clean shutdown (Ctrl-C / SIGTERM).

---

## API reference

### Broker

| Method   | Path                    | Description                              |
|----------|-------------------------|------------------------------------------|
| `GET`    | `/health`               | Returns `{"status":"ok"}`                |
| `POST`   | `/api/v1/nodes`         | Register a node (body: `RegisterRequest`) |
| `GET`    | `/api/v1/nodes`         | List all registered nodes                |
| `DELETE` | `/api/v1/nodes/:id`     | Deregister a node                        |

**`RegisterRequest`**
```json
{
  "name": "my-node",
  "url": "http://192.168.1.42:4000",
  "capabilities": [
    { "name": "text-generation", "description": "LLM inference" }
  ]
}
```

**`RegisterResponse`**
```json
{ "id": "<uuid>", "url": "http://192.168.1.42:4000" }
```

### Node

| Method | Path              | Description                          |
|--------|-------------------|--------------------------------------|
| `GET`  | `/health`         | Returns `{"status":"ok"}`            |
| `POST` | `/api/v1/infer`   | Run inference (body: `InferRequest`) |

**`InferRequest`**
```json
{ "capability": "text-generation", "input": { "prompt": "Hello!" } }
```

**`InferResponse`**
```json
{ "capability": "text-generation", "output": { "text": "[generated] Hello!" } }
```

Built-in capabilities:

| Name               | Input fields   | Output fields                      |
|--------------------|----------------|------------------------------------|
| `text-generation`  | `prompt`       | `text` (echoed with prefix)        |
| `echo`             | anything       | same as input                      |

---

## Configuration (node)

All settings are controlled via environment variables:

| Variable          | Default                        | Description                           |
|-------------------|--------------------------------|---------------------------------------|
| `NODE_NAME`       | `ai-node`                      | Human-readable node name              |
| `NODE_PORT`       | `4000`                         | Port the node API binds to            |
| `NODE_PUBLIC_URL` | `http://localhost:<NODE_PORT>` | URL advertised to the broker          |
| `BROKER_URL`      | `http://localhost:3000`        | Base URL of the broker                |

Example – run a second node on port 4001:

```bash
NODE_NAME=node-2 NODE_PORT=4001 NODE_PUBLIC_URL=http://localhost:4001 cargo run --bin node
```

