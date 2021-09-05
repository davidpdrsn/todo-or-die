use anyhow::{Context as _, Result};
use chrono::prelude::*;
use hyper::{
    body::Bytes,
    client::{connect::dns::GaiResolver, HttpConnector},
    header::HeaderValue,
    header::USER_AGENT,
    Body, Client, Request, Response,
};
use hyper_rustls::HttpsConnector;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    path::PathBuf,
};
use tokio::runtime::Runtime;

pub(crate) fn request<T>(
    // the request body isn't used in the cache key, so require it to be `()` so
    // we can guarantee that its empty
    request: Request<()>,
) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    RUNTIME.block_on(async move {
        let mut request = request.map(|_| Body::empty());

        request
            .headers_mut()
            .insert(USER_AGENT, HeaderValue::from_static("todo-or-die"));

        let hash = hash_request(&request);

        let response = if let Some(cached_response) =
            cached_response(&hash).context("Failed to read cached response")?
        {
            cached_response
        } else {
            let response = http_client()
                .request(request)
                .await
                .context("HTTP request to failed")?;

            let (parts, body) = response.into_parts();
            let body = hyper::body::to_bytes(body)
                .await
                .context("Failed to read response")?;
            let response = Response::from_parts(parts, body);

            cache_response(hash, &response).context("Failed to cache response")?;

            response
        };

        if !response.status().is_success() {
            let body = String::from_utf8_lossy(response.body());
            anyhow::bail!(
                "Received non-success response. status={}, body={:?}",
                response.status(),
                body
            );
        }

        let value =
            serde_json::from_slice::<T>(&*response.body()).context("Failed to parse response")?;
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct RequestHash(String);

fn hash_request(request: &Request<Body>) -> RequestHash {
    let mut hasher = DefaultHasher::new();
    format!("{:?}", request).hash(&mut hasher);
    let hash = hasher.finish();
    RequestHash(hash.to_string())
}

fn cached_response(hash: &RequestHash) -> Result<Option<Response<Bytes>>> {
    let path = cache_dir_path()?.join(&hash.0);

    let data = match std::fs::read(&path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };

    if let Some(response) = deserialize_response(data)? {
        Ok(Some(response))
    } else {
        std::fs::remove_file(&path)?;
        Ok(None)
    }
}

fn cache_response(hash: RequestHash, response: &Response<Bytes>) -> Result<()> {
    let path = cache_dir_path()?.join(hash.0);
    let bytes = serialize_response(response)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

fn serialize_response(response: &Response<Bytes>) -> Result<Vec<u8>> {
    let headers = response
        .headers()
        .iter()
        .map(|(key, value)| (key.as_str().to_string(), value.as_bytes().to_vec()))
        .collect();

    let response = SerializedResponse {
        status: response.status().as_u16(),
        headers,
        body: response.body().to_vec(),
        expires_at: Local::now() + cache_ttl(),
    };

    Ok(serde_json::to_vec(&response)?)
}

fn cache_ttl() -> chrono::Duration {
    (|| {
        let var = std::env::var("TODO_OR_DIE_HTTP_CACHE_TTL_SECONDS")?;
        let sec = var.parse()?;
        Ok::<_, anyhow::Error>(chrono::Duration::seconds(sec))
    })()
    .unwrap_or_else(|_| chrono::Duration::hours(1))
}

fn deserialize_response(data: Vec<u8>) -> Result<Option<Response<Bytes>>> {
    let response = serde_json::from_slice::<SerializedResponse>(&data)
        .context("Failed to deserialize cached HTTP response")?;

    let expires_at = response.expires_at.timestamp();
    let now = Local::now().timestamp();
    if now > expires_at {
        return Ok(None);
    }

    let status = hyper::StatusCode::from_u16(response.status)?;

    let headers = response
        .headers
        .iter()
        .map(|(key, value)| {
            let key = hyper::header::HeaderName::from_bytes(key.as_bytes())?;
            let value = HeaderValue::from_bytes(value)?;
            Ok::<_, anyhow::Error>((key, value))
        })
        .collect::<Result<hyper::HeaderMap>>()
        .context("Failed to build header map")?;

    let body = Bytes::copy_from_slice(&response.body);

    let mut out = Response::new(body);
    *out.status_mut() = status;
    *out.headers_mut() = headers;
    Ok(Some(out))
}

#[derive(Serialize, Deserialize)]
struct SerializedResponse {
    status: u16,
    headers: HashMap<String, Vec<u8>>,
    body: Vec<u8>,
    expires_at: DateTime<Local>,
}

fn cache_dir_path() -> Result<PathBuf> {
    let todo_or_die_version = env!("CARGO_PKG_VERSION");
    let path = std::env::temp_dir().join(format!("todo_or_die_{}_cache", todo_or_die_version));
    std::fs::create_dir_all(&path).context("Failed to create dir to store HTTP caches")?;
    Ok(path)
}
