/// Represents a single backend server.
///
/// A backend is just a server where the load balancer forwards incoming client requests.
pub struct Backend {
    /// Address of the backend server
    pub addr: String,
}

/// A collection of backend servers.
/// The load balancer will choose one of these servers when forwarding requests.
pub struct BackendPool {
    /// List of all available backend servers
    pub backends: Vec<Backend>,
}

impl BackendPool {
    /// Creates a backend pool from a list of backend addresses.
    /// Each address is converted into a `Backend` and stored inside the pool.
    pub fn from_config(addrs: &[String]) -> Self {
        BackendPool {
            backends: addrs
                .iter()
                .map(|a| Backend { addr: a.clone() })
                .collect(),
        }
    }
}