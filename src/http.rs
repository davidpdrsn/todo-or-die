use anyhow::{Context as _, Result};
use hyper::{
    client::{connect::dns::GaiResolver, HttpConnector},
    header::HeaderValue,
    header::USER_AGENT,
    Body, Client, Request,
};
use hyper_rustls::HttpsConnector;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

pub(crate) fn request<T>(mut request: Request<Body>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    RUNTIME.block_on(async move {
        request
            .headers_mut()
            .insert(USER_AGENT, HeaderValue::from_static("todo-or-die"));

        let response = http_client()
            .request(request)
            .await
            .context("HTTP request to failed")?;

        let status = response.status();
        if !status.is_success() {
            let body = hyper::body::to_bytes(response)
                .await
                .context("Failed to read response")?;
            let body = String::from_utf8_lossy(&body);
            anyhow::bail!(
                "Received non-success response. status={}, body={:?}",
                status,
                body
            );
        }

        let body = hyper::body::to_bytes(response)
            .await
            .context("Failed to read response")?;
        let value = serde_json::from_slice::<T>(&body).context("Failed to parse response")?;

        Ok(value)
    })
}

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
});

type HyperTlsClient = Client<HttpsConnector<HttpConnector<GaiResolver>>, Body>;

fn http_client() -> &'static HyperTlsClient {
    static CLIENT: Lazy<HyperTlsClient> = Lazy::new(|| {
        let mut tls = rustls::ClientConfig::new();
        tls.set_protocols(&["h2".into(), "http/1.1".into()]);
        tls.root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        let mut http = hyper::client::HttpConnector::new();
        http.enforce_http(false);

        hyper::Client::builder().build::<_, Body>(hyper_rustls::HttpsConnector::from((http, tls)))
    });

    &*CLIENT
}
