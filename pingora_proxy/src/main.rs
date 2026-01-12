use async_trait::async_trait;
use pingora::prelude::*;
use std::sync::Arc;

pub struct LB {
    lb: Arc<LoadBalancer<RoundRobin>>,
}

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let upstream = self.lb.select(b"", 256).unwrap();
        // println!("Forwarding to: {:?}", upstream);

        // Connect to upstream (API container)
        // bool: true = TLS, false = Plain HTTP
        let peer = Box::new(HttpPeer::new(upstream, false, "api".to_string()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Pass the original Host header or set it to upstream?
        // Usually good to preserve or set explicitly.
        // For now we keep it simple.
        upstream_request
            .insert_header("X-Forwarded-By", "Pingora")
            .unwrap();
        Ok(())
    }
}

fn main() {
    env_logger::init();

    // Create a Server
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    // Define Upstreams (The 'api' service in Docker Compose)
    let upstreams = LoadBalancer::try_from_iter(["api:8080"]).unwrap();

    let mut lb = http_proxy_service(
        &my_server.configuration,
        LB {
            lb: Arc::new(upstreams),
        },
    );

    // Listen on Port 80 (inside container)
    lb.add_tcp("0.0.0.0:80");

    my_server.add_service(lb);
    my_server.run_forever();
}
