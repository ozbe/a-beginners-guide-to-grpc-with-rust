use hello::say_server::{Say, SayServer};
use hello::{SayRequest, SayResponse};
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

mod hello;

#[derive(Default)]
pub struct MySay {}

#[tonic::async_trait]
impl Say for MySay {
    type SendStreamStream = mpsc::Receiver<Result<SayResponse, Status>>;

    async fn send_stream(
        &self,
        request: Request<SayRequest>,
    ) -> Result<Response<Self::SendStreamStream>, Status> {
        let (mut tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            for _ in 0..4 {
                tx.send(Ok(SayResponse {
                    message: format!("hello"),
                }))
                .await;
            }
        });
        Ok(Response::new(rx))
    }

    async fn send(&self, request: Request<SayRequest>) -> Result<Response<SayResponse>, Status> {
        Ok(Response::new(SayResponse {
            message: format!("hello {}", request.get_ref().name),
        }))
    }

    async fn receive_stream(
        &self,
        request: Request<tonic::Streaming<SayRequest>>,
    ) -> Result<Response<SayResponse>, Status> {
        let mut stream = request.into_inner();
        let mut message = String::from("");
        while let Some(req) = stream.message().await? {
            message.push_str(&format!("Hello {}\n", req.name))
        }
        Ok(Response::new(SayResponse { message }))
    }

    type BidirectionalStream = mpsc::Receiver<Result<SayResponse, Status>>;

    async fn bidirectional(
        &self,
        request: Request<tonic::Streaming<SayRequest>>,
    ) -> Result<Response<Self::BidirectionalStream>, Status> {
        let mut streamer = request.into_inner();
        let (mut tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            while let Some(req) = streamer.message().await.unwrap() {
                tx.send(Ok(SayResponse {
                    message: format!("hello {}", req.name),
                }))
                .await;
            }
        });
        Ok(Response::new(rx))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let say = MySay::default();
    let ser = SayServer::with_interceptor(say, interceptor);
    let cert = include_str!("../server.pem");
    let key = include_str!("../server.key");
    let id = tonic::transport::Identity::from_pem(cert.as_bytes(), key.as_bytes());
    let s = include_str!("../my_ca.pem");
    let ca = tonic::transport::Certificate::from_pem(s.as_bytes());
    let tls = tonic::transport::ServerTlsConfig::new()
        .identity(id)
        .client_ca_root(ca);

    println!("Server listening on {}", addr);

    Server::builder()
        .tls_config(tls)
        .add_service(ser)
        .serve(addr)
        .await?;

    Ok(())
}

fn interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let token = match req.metadata().get("authorization") {
        Some(token) => token.to_str(),
        None => return Err(Status::unauthenticated("Token not found")),
    };
    if let Ok("secret_token") = token {
        Ok(req)
    } else {
        Err(Status::unauthenticated("Invalid token"))
    }
}
