use hyper::{Client, Server, Request, Body};
use anyhow::*;
use std::net::SocketAddr;
use hyper::service::{make_service_fn, service_fn};
use std::sync::Arc;


fn mutate_request(req: &mut Request<Body>) -> Result<()>{
  for key in ["content-length", "transfer-encoding", "accept-encoding", "content-encoding"].iter() {
    req.headers_mut().remove(*key);

    let uri = req.uri();
    let uri_str = match uri.query() {
      None => format!("https://www.snoyman.com{}", uri.path()),
      Some(query) => format!("https://www.snoyman.com{}?{}", uri.path(), query),
    };
    *req.uri_mut() = uri_str.parse().context("Mutate uri")?;
  }
  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  let https =  hyper_rustls::HttpsConnector::with_native_roots();
  let client: Client<_, hyper::Body> = Client::builder().build(https);
  let client: Arc<Client<_, hyper::Body>> = Arc::new(client);
  let addr  = SocketAddr::from(([0, 0, 0, 0], 3094));


  let make_svc = make_service_fn(move |_| {
    let client = Arc::clone(&client);
    async move {
      Ok::<_, Error>(service_fn(move |mut req| {
        let client = Arc::clone(&client);
        async move {
          mutate_request(&mut req)?;
          client.request(req).await.context("Making request to backend")
        }
      }))
    }
  });
  Server::bind(&addr).serve(make_svc).await.context("Running Server")?;
  Ok::<(), anyhow::Error>(())
}
