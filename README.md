# A beginners guide to gRPC with Rust

Run-through of [A beginners guide to gRPC with Rust](https://dev.to/anshulgoyal15/a-beginners-guide-to-grpc-with-rust-3c7o).

## Setup

```bash
$ openssl genrsa -des3 -out my_ca.key 2048
$ openssl req -x509 -new -nodes -key my_ca.key -sha256 -days 1825 -out my_ca.pem

# server
$ openssL genrsa -out server.key 2048
$ openssl req -new -sha256 -key server.key -out server.csr
$ cat >> server.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
EOF
$ openssl x509 -req -in server.csr -CA my_ca.pem -CAkey my_ca.key -CAcreateserial -out server.pem -days 1825 -sha256 -extfile server.ext

# client
$ openssL genrsa -out client.key 2048
$ openssl req -new -sha256 -key client.key -out client.csr
$ cat >> client.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
EOF
$ openssl x509 -req -in client.csr -CA my_ca.pem -CAkey my_ca.key -CAcreateserial -out client.pem -days 1825 -sha256 -extfile client.ext

```

## Run

### Server
```bash
$ cargo run --bin server
```

### Client
```bash
$ cargo run --bin client -- [METHOD]
```

**METHOD**
- `receive-stream`
- `send-stream`
- `bidirectional`
- default: `send` 