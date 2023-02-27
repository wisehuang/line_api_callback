use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bytes::Bytes;
use config::builder::DefaultState;
use config::ConfigBuilder;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::{Filter, Rejection, Reply};

pub fn get_channel_secret(config_builder: ConfigBuilder<DefaultState>) -> String {
    let channel_secret: String = match config_builder.build() {
        Ok(config) => config
            .get::<String>("channel.secret")
            .expect("Missing channel.secret in config file"),
        Err(e) => {
            panic!("{}", e);
        }
    };
    channel_secret
}

pub async fn parse_request_handler(
    x_line_signature: String,
    body: Bytes,
    channel_secret: Arc<Mutex<String>>,
) -> Result<impl Reply, Rejection> {
    let channel_secret = match channel_secret.lock() {
        Ok(secret) => secret,
        Err(poisoned) => poisoned.into_inner(),
    };

    log::info!("channel secret: {}", channel_secret);

    let encoded_body = generate_signature(&channel_secret, &body);

    log::info!("encoded body: {}", encoded_body);
    log::info!("x-line-signature: {:?}", x_line_signature);
    log::info!(
        "body content: {}",
        String::from_utf8(body.to_vec()).unwrap()
    );

    if encoded_body == x_line_signature {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": true})),
            warp::http::StatusCode::OK,
        ))
    } else {
        let error_msg = json!({"success": false, "error": "Invalid signature"});
        let response = warp::reply::with_status(
            warp::reply::json(&error_msg),
            warp::http::StatusCode::BAD_REQUEST,
        );
        Ok(response)
    }
}

fn generate_signature(channel_secret: &str, body: &[u8]) -> String {
    let mut hmac_sha256 =
        Hmac::<Sha256>::new_from_slice(channel_secret.as_bytes()).expect("Failed to create HMAC");
    hmac_sha256.update(&body);

    BASE64.encode(hmac_sha256.finalize().into_bytes())
}

pub fn with_channel_secret(
    channel_secret: Arc<Mutex<String>>,
) -> impl Filter<Extract = (Arc<Mutex<String>>,), Error = Infallible> + Clone {
    warp::any().map(move || channel_secret.clone())
}
