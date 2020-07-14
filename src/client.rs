use futures::stream::iter;
use hello::say_client::SayClient;
use hello::SayRequest;
use tonic::Request;
mod hello;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert = include_str!("../client.pem");
    let key = include_str!("../client.key");
    let id = tonic::transport::Identity::from_pem(cert.as_bytes(), key.as_bytes());
    let s = include_str!("../my_ca.pem");
    let ca = tonic::transport::Certificate::from_pem(s.as_bytes());
    let tls = tonic::transport::ClientTlsConfig::new()
        .domain_name("localhost")
        .identity(id)
        .ca_certificate(ca);
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .tls_config(tls)
        .connect()
        .await?;
    let token = get_token();
    let client = SayClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::from_str(&token).unwrap(),
        );
        Ok(req)
    });

    match std::env::args().next().as_deref() {
        Some("receive-stream") => receive_stream(client),
        Some("send-stream") => send_stream(client),
        Some("bidirectional") => bidirectional(client),
        _ => send(client),
    }
    .await?;

    Ok(())
}

async fn send(
    client: SayClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(SayRequest {
        name: String::from("anshul"),
    });
    let response = client.send(request).await?.into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}

async fn send_stream(
    client: SayClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(SayRequest {
        name: String::from("anshul"),
    });

    let mut response = client.send_stream(request).await?.into_inner();
    while let Some(res) = response.message().await? {
        println!("NOTE = {:?}", res);
    }

    Ok(())
}

async fn receive_stream(
    client: SayClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let response = client.receive_stream(request).await?.into_inner();
    println!("RESPONSE=\n{}", response.message);

    Ok(())
}

async fn bidirectional(
    client: SayClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut response = client.bidirectional(request).await?.into_inner();
    while let Some(res) = response.message().await? {
        println!("NOTE = {:?}", res);
    }

    Ok(())
}

fn get_token() -> String {
    "secret_token".to_string()
}
