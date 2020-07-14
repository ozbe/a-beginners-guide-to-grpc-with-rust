use futures::stream::iter;
use hello::say_client::SayClient;
use hello::SayRequest;
use tonic::Request;
mod hello;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // getting certificate from disk
    let cert = include_str!("../client.pem");
    let key = include_str!("../client.key");
    // creating identify from key and certificate
    let id = tonic::transport::Identity::from_pem(cert.as_bytes(), key.as_bytes());
    // importing our certificate for CA
    let s = include_str!("../my_ca.pem");
    // converting it into a certificate
    let ca = tonic::transport::Certificate::from_pem(s.as_bytes());
    // telling our client what is the identity of our server
    let tls = tonic::transport::ClientTlsConfig::new()
        .domain_name("localhost")
        .identity(id)
        .ca_certificate(ca);
    // connecting with tls
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .tls_config(tls)
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
