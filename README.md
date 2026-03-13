# rust-lb

`rust-lb` is a personal learning project focused on building a **high-performance asynchronous TCP load balancer in Rust** from scratch.

The goal of this project is not just to produce a working proxy, but to **deeply understand how modern networking infrastructure works internally**. Instead of relying on high-level frameworks, the implementation focuses on learning the underlying systems concepts such as asynchronous execution, TCP networking, connection management, rate limiting, and proxy architectures.

This project is being developed incrementally as a systems programming exercise while exploring Rust’s async ecosystem and Tokio runtime and diving into networking.

---

# Current Implementation

The current version implements the foundations of a simple **asynchronous TCP proxy**.

Core functionality includes:

* Async TCP server using the Tokio runtime
* Accept loop for handling incoming client connections
* Backend connection establishment
* **Bidirectional stream proxying** between client and backend
* Per-IP **Token Bucket rate limiting**
* Connection timeouts to prevent long-running idle connections
* Graceful shutdown support
* Structured project layout separating proxy logic, configuration, errors, and rate limiting

At the moment the proxy behaves as a simple TCP forwarder:

```
client  <---->  rust-lb  <---->  backend
```

The proxy accepts client connections, optionally rate-limits them, forwards traffic to a backend server, and transparently relays data in both directions.

---

# Project Structure

```
src/
│
├── main.rs          # Application entry point and runtime setup
├── proxy.rs         # TCP accept loop and proxy logic
├── rate_limit.rs    # Token bucket rate limiter
├── config.rs        # Configuration loading
└── error.rs         # Application error types
```

The codebase is intentionally organized to keep networking logic, configuration, and error handling modular as the system grows.

---

# Running the Proxy

Create a configuration file:

```toml
listen_addr = "127.0.0.1:8080"

backends = [
  "127.0.0.1:9000"
]
```

Start the proxy with:

```
cargo run
```

Once running, the proxy listens for incoming TCP connections and forwards them to the configured backend server.

---

# Motivation

This project exists primarily as a **hands-on exploration of systems programming**.

By building a networking system piece-by-piece, the project aims to explore topics such as:

* asynchronous programming in Rust
* TCP networking internals
* proxy and load balancer design
* concurrency and shared state
* performance considerations in network services

The long-term intention is to gradually evolve this prototype into a **feature-rich and performant load balancer**, while documenting the learning process along the way.
