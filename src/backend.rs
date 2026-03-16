//not in use yet

#![allow(dead_code)]
pub struct Backend {
    pub addr: String,
}

pub struct BackendPool {
    pub backends: Vec<Backend>,
}

impl BackendPool {
    pub fn from_config(addrs: &[String]) -> Self {
        BackendPool {
            backends: addrs.iter().map(|a| Backend { addr: a.clone() }).collect(),
        }
    }
}
