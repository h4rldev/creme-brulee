use creme_brulee::{
    IoError, IoResult,
    config::{Config, Level},
};
use openssl::ssl::{AlpnError, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use quinn::ServerConfig;
use std::{fs, /* io,*/ sync::Arc};
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::{
    field::MakeExt,
    fmt::{Subscriber, format::debug_fn},
};

use xitca_web::{
    App,
    body::ResponseBody,
    handler::handler_service,
    http::{Response, WebResponse},
    middleware::{compress::Compress, decompress::Decompress},
    route::get,
};

mod creme_brulee;

async fn index() -> Result<Response<ResponseBody>, IoError> {
    let response = WebResponse::builder()
        .status(200)
        .body("Hello, World!".into())
        .unwrap();

    Ok(response)
}

fn main() -> IoResult<()> {
    let formatter =
        debug_fn(|writer, field, value| write!(writer, "{field}: {value:?}")).delimited(",");

    let config = Config::load().unwrap_or_else(|e| panic!("failed to load config: {e}"));

    let level: Level = config.logging().level.clone().into();

    Subscriber::builder()
        .with_max_level(level.0)
        .fmt_fields(formatter)
        .with_ansi(true)
        .init();

    // construct server endpoints, and potentially file server.
    let app = App::new().at("/", get(handler_service(index)));

    // wrap the app with compression middleware.
    let with_middleware = app.enclosed(Compress).enclosed(Decompress);

    // bing and start the server.
    let mut server = with_middleware.serve();

    let bind = config.network().bind.clone();
    let quic_ip = bind.split(':').next().unwrap();
    let quic_port = config.network().quic_port.unwrap_or(8081);
    let quic_bind = format!("{quic_ip}:{quic_port}");

    if config.tls().enable {
        server = server.bind_openssl(&bind, h2_config()?)?;
    }

    if config.tls().quic {
        server = server.bind_h3(&quic_bind, h3_config()?)?;
    }

    if !config.tls().enable && !config.tls().quic {
        server = server.bind(&bind)?;
    }

    server.run().wait()
}

fn h2_config() -> IoResult<SslAcceptorBuilder> {
    // set up openssl and alpn protocol.
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file(".certs/localhost.key", SslFiletype::PEM)?;
    builder.set_certificate_chain_file(".certs/localhost.crt")?;

    builder.set_alpn_select_callback(|_, protocols| {
        const H2: &[u8] = b"\x02h2";
        const H11: &[u8] = b"\x08http/1.1";

        if protocols.windows(3).any(|window| window == H2) {
            Ok(b"h2")
        } else if protocols.windows(9).any(|window| window == H11) {
            Ok(b"http/1.1")
        } else {
            Err(AlpnError::NOACK)
        }
    });

    builder.set_alpn_protos(b"\x08http/1.1\x02h2")?;

    Ok(builder)
}

fn h3_config() -> IoResult<ServerConfig> {
    let cert = fs::read(".certs/localhost.crt")?;
    let key = fs::read(".certs/localhost.key")?;

    let key = rustls_pemfile::pkcs8_private_keys(&mut &*key)
        .next()
        .unwrap()
        .unwrap();
    let key = quinn::rustls::pki_types::PrivateKeyDer::from(key);

    let cert = rustls_pemfile::certs(&mut &*cert)
        .collect::<Result<_, _>>()
        .unwrap();

    let mut config = quinn::rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .unwrap();

    config.alpn_protocols = vec![b"h3".to_vec()];

    let config = quinn::crypto::rustls::QuicServerConfig::try_from(config).unwrap();

    Ok(ServerConfig::with_crypto(Arc::new(config)))
}
