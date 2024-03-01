use axum::{RequestExt, Router};
use axum::{body:: Body, http::{method::Method, uri::{self, PathAndQuery}, HeaderMap, Request, Response, Uri}};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crate::backend::pool::ServerPool;


#[derive(Clone)]
pub struct LoadBalancer {
    host: String,
    port: u32,
    app : Option<Router>,
    pool: ServerPool
}

impl LoadBalancer { 
    pub fn new(host: String , port : u32, pool : ServerPool) -> Self { 
        LoadBalancer {host, port, app: None, pool }
    } 

    fn addr(&self) -> SocketAddr{
        let socket_addr = SocketAddr::from_str(format!("{}:{}",self.host, self.port ).as_str()).map_err(|err|  "could not parse addr").unwrap();
        
        socket_addr
    }

    pub async fn start(&mut self) {
        self.initRouter().await;
        if let Some(router) = &self.app {
            println!("hello");
            axum::Server::bind(&self.addr()).serve(router.clone().into_make_service()).await.map_err(|_| "error creating socket");
        }
    }

    pub async fn initRouter(&mut self)  { 
        let lb = Arc::new(Mutex::new(self.clone()));
        //println!("here");

        let app = axum::Router::new().fallback( move|mut request : Request<Body>| async move {
            println!("right here");
            let req_uri : Uri= request.extract_parts().await.unwrap();
            let url = Arc::clone(&lb).lock().unwrap().pool.next_available_server(req_uri.to_string()).unwrap().addr();
            //let proxy_origin = self.pool.next_available_server("vsdfsd".to_string()).unwrap().addr();
            let res = proxy_request(request, url).await.unwrap();
            res
        });
        if self.app.is_none() {
            self.app = Some(app);
        }
    }

    

}


pub async fn proxy_request(mut request: Request<Body>, backendUrl : String) -> Result<Response<Body> , String> {
    let uri : Uri = request.extract_parts().await.unwrap(); 
    let method : Method = request.extract_parts().await.unwrap(); 
    let req_headers: HeaderMap = request.extract_parts().await.unwrap();

    let data = request.into_body();
    let req_body = reqwest::Body::try_from(data).unwrap();
    let client = reqwest::Client::new();
    let p_and_q = uri.path_and_query().cloned().unwrap_or_else(|| PathAndQuery::from_static("/"));
    let url  = uri::Builder::new().scheme("http")
        .authority(backendUrl)
        .path_and_query(p_and_q.clone())
        .build()
        .map_err(|_| "could not build url")?;
    println!("url : {}, method : {}, req_headers :{:#?}", url.clone(), method.clone(), req_headers.clone());
    let  (status, res_headers, bytes) = client.request(method, url.clone().to_string()).headers(req_headers).
        body(req_body)
        .send().await
        .map_err(|err| err.to_string()).map(|r| (r.status(), r.headers().clone(), r.bytes()))?;
    let body = bytes.await.unwrap();
    let mut res = Response::new(axum::body::Body::from(body));
    let headers = res.headers_mut();
    headers.extend(res_headers.clone().iter().map(|(name, val)| (name.clone(), val.clone())));
    *res.status_mut() = status;
    println!("res : {:#?}", res);
    Ok(res)
    
}
