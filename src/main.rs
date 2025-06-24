use axum::Router;
use creme_brulee::{
    IoResult,
    cli::init,
    config::{Level, string_to_ip},
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::{
    field::MakeExt,
    fmt::{Subscriber, format::debug_fn},
};

mod creme_brulee;
use axum_server::tls_rustls::RustlsConfig;

#[tokio::main]
async fn main() -> IoResult<()> {
    let formatter =
        debug_fn(|writer, field, value| write!(writer, "{field}: {value:?}")).delimited(",");

    let config = init();
    let level: Level = config.logging().level.clone().into();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    Subscriber::builder()
        .with_max_level(level.0)
        .fmt_fields(formatter)
        .with_ansi(true)
        .init();

    let root = config
        .site()
        .root
        .clone()
        .unwrap_or_else(|| panic!("invalid root path"));
    let error_page = config
        .site()
        .error
        .clone()
        .unwrap_or_else(|| panic!("invalid error page"));

    debug!("root: {root:?}");
    debug!("error page: {error_page:?}");

    let serve_dir = ServeDir::new(root).not_found_service(ServeFile::new(error_page));

    let app = Router::new()
        .fallback_service(serve_dir)
        //
        // This adds compression and decompression to the request and response
        // body streams, don't remove it!
        //
        .layer(
            ServiceBuilder::new()
                .layer(tower_http::decompression::RequestDecompressionLayer::new())
                .layer(
                    tower_http::compression::CompressionLayer::new()
                        .br(true)
                        .zstd(true),
                ),
        );

    let ip = string_to_ip(&config.network().ip).unwrap_or_else(|e| panic!("invalid ip: {e}"));
    let addr = SocketAddr::from((ip, config.network().port));

    if config.tls().enable {
        let cert_path = config
            .tls()
            .cert
            .clone()
            .unwrap_or_else(|| panic!("invalid cert path"));
        let key_path = config
            .tls()
            .key
            .clone()
            .unwrap_or_else(|| panic!("invalid key path"));

        let tls_config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
        info!("serving https on {addr}");
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await
    } else {
        info!("serving http on {addr}");
        axum_server::bind(addr).serve(app.into_make_service()).await
    }
}
