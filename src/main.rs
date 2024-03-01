pub mod lb_utils;
mod backend;
use backend::pool::{BackendServer, ServerPool};
use lb_utils::lb::LoadBalancer;




#[tokio::main]
async fn main() -> Result<(), String>{
    let mut nodes = Vec::new();
    let server1 = BackendServer::new("localhost:8000".to_string(), 50, true);
    let server2 = BackendServer::new("localhost:8001".to_string(), 50, true);
    nodes.push(server1);
    nodes.push(server2);
    let pool = ServerPool::new(nodes);  
    let mut lb = LoadBalancer::new("0.0.0.0".to_string(), 8080, pool);
    let lb_handle = tokio::spawn(async move {
        lb.start().await;
    });

    // Wait for a Ctrl+C signal to gracefully shutdown the server
    tokio::signal::ctrl_c().await.map_err(|err| err.to_string())?;

    // Await the load balancer handle to complete
    lb_handle.await.map_err(|err| err.to_string())?;

    Ok(())
}
