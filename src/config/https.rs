use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

pub fn config_https() -> SslAcceptorBuilder {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("/app/127.0.0.1-key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("/app/127.0.0.1.pem").unwrap();

    builder
}
