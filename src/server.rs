use hello::say_server::{Say, SayServer};
use hello::{SayRequest, SayResponse};
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};
mod hello;
#[derive(Default)]
pub struct MySay {}
#[tonic::async_trait]
impl Say for MySay {
    // Specify the output of rpc call
    type SendStreamStream = mpsc::Receiver<Result<SayResponse, Status>>;
    // implementation for rpc call
    async fn send_stream(
        &self,
        request: Request<SayRequest>,
    ) -> Result<Response<Self::SendStreamStream>, Status> {
        // creating a queue or channel
        let (mut tx, rx) = mpsc::channel(4);
        // creating a new task
        tokio::spawn(async move {
            // looping and sending our response using stream
            for _ in 0..4 {
                // sending response to our channel
                tx.send(Ok(SayResponse {
                    message: format!("hello"),
                }))
                .await;
            }
        });
        // returning our reciever so that tonic can listen on reciever and send the response to client
        Ok(Response::new(rx))
    }
    async fn send(&self, request: Request<SayRequest>) -> Result<Response<SayResponse>, Status> {
        Ok(Response::new(SayResponse {
            message: format!("hello {}", request.get_ref().name),
        }))
    }
    // create a new rpc to receive a stream
    async fn receive_stream(
        &self,
        request: Request<tonic::Streaming<SayRequest>>,
    ) -> Result<Response<SayResponse>, Status> {
        // converting request into stream
        let mut stream = request.into_inner();
        let mut message = String::from("");
        // listening on stream
        while let Some(req) = stream.message().await? {
            message.push_str(&format!("Hello {}\n", req.name))
        }
        // returning response
        Ok(Response::new(SayResponse { message }))
    }
    // defining return stream
    type BidirectionalStream = mpsc::Receiver<Result<SayResponse, Status>>;
    async fn bidirectional(
        &self,
        request: Request<tonic::Streaming<SayRequest>>,
    ) -> Result<Response<Self::BidirectionalStream>, Status> {
        // converting request in stream
        let mut streamer = request.into_inner();
        // creating queue
        let (mut tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            // listening on request stream
            while let Some(req) = streamer.message().await.unwrap() {
                // sending data as soon it is available
                tx.send(Ok(SayResponse {
                    message: format!("hello {}", req.name),
                }))
                .await;
            }
        });
        // returning stream as receiver
        Ok(Response::new(rx))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let say = MySay::default();
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(SayServer::new(say))
        .serve(addr)
        .await?;
    Ok(())
}
