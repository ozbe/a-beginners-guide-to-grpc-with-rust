use futures::stream::iter;
use hello::say_client::SayClient;
use hello::SayRequest;
use tonic::Request;
mod hello;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;
    let token = get_token(); // an method to get token can be a rpc call etc.
    let mut client = SayClient::with_interceptor(channel, move |mut req: Request<()>| {
        // adding token to request.
        req.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::from_str(&token).unwrap(),
        );
        Ok(req)
    });

    // creating a client
    let request = tonic::Request::new(iter(vec![
        SayRequest {
            name: String::from("anshul"),
        },
        SayRequest {
            name: String::from("rahul"),
        },
        SayRequest {
            name: String::from("vijay"),
        },
    ]));
    // calling rpc
    let mut response = client.bidirectional(request).await?.into_inner();
    // listening on the response stream
    while let Some(res) = response.message().await? {
        println!("NOTE = {:?}", res);
    }
    Ok(())
}

fn get_token() -> String {
  "token".to_string()
}