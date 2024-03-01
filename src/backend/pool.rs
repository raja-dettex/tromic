use std::{collections::HashMap, hash::Hasher};

use fnv::FnvHasher;


#[derive( PartialEq, Debug, Eq, PartialOrd, Ord, Clone)]
pub struct BackendServer {         
    pub addr : String,
    pub weight : usize,
    pub isHealthy : bool
}



impl BackendServer {
    pub fn new(addr : String, weight: usize, isHealthy: bool ) -> Self {
        BackendServer{addr, weight, isHealthy}
    }

    pub fn addr(&self) -> String {
        let addr = &self.addr;
        addr.to_string()
    }
}


#[derive(Clone)]
pub struct ServerPool {
    pub servers : HashMap<u64, BackendServer>,
    nodes : Vec<BackendServer>
}

impl ServerPool {
    pub fn new(nodes : Vec<BackendServer>) -> Self {
        let mut servers = HashMap::new();
        for server in &nodes {
            let hash = hash(server.addr.clone());
            servers.insert(hash, server.clone());
        }
        ServerPool{servers, nodes}
    }
    pub fn next_available_server(&self, key : String) -> Option<BackendServer> {
        let hash = hash(key.clone());
        println!("key : {} , hash : {}", key, hash);
        let mut greater_nodes: Vec<BackendServer> = self.servers.iter()
            .filter(|(node_hash, _)| *node_hash >= &hash)
            .map(|(_, node) | node.clone()).collect();
        if greater_nodes.is_empty() {
            greater_nodes = self.nodes.clone();
        } 
        greater_nodes.sort();
        Some(greater_nodes[0].clone())       
    }
}

pub fn hash(key : String) -> u64 {
    let mut hasher = FnvHasher::default();
    hasher.write(key.as_bytes());
    hasher.finish()
}