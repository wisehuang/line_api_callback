use std::sync::{Arc, Mutex};
use warp::{Filter, Rejection, Reply};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde_json::json;
use bytes::Bytes;
use std::convert::Infallible;
use config::{Config, File, FileFormat};
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    let config_builder = Config::builder()
    .add_source(File::new("config.toml", FileFormat::Toml));

    let channel_secret:String = match config_builder.build() {
        Ok(config) => {
            config.get::<String>("channel.secret").expect("Missing channel.secret in config file")            
        },
        Err(e) => {
            panic!("{}", e);
        }
    };

    let parse_request_route = warp::post().and(warp::path("verify"))
        .and(warp::header::<String>("x-line-signature"))
        .and(warp::body::bytes())
        .and(with_channel_secret(Arc::new(Mutex::new(channel_secret))))
        .and_then(parse_request_handler);

    let test_route = warp::get().and(warp::path("hello")).map(|| {        
        Ok(warp::reply::json(&json!({"success": true})))
    });

    let log_filter = warp::log("line_api_callback");

    let routes = parse_request_route.or(test_route).with(log_filter);
    
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn parse_request_handler(
    x_line_signature: String,
    body: Bytes,
    channel_secret: Arc<Mutex<String>>,
) -> Result<impl Reply, Rejection> {
    let channel_secret = channel_secret.lock().unwrap();

    log::info!("channel secret: {}", channel_secret);

    let encoded_body = generate_signature(&channel_secret, &body);

    log::info!("encoded body: {}", encoded_body);
    log::info!("x-line-signature: {:?}", x_line_signature);
    log::info!("body content: {}", String::from_utf8(body.to_vec()).unwrap());

    if encoded_body == x_line_signature {
        Ok(warp::reply::with_status(warp::reply::json(&json!({"success": true})), warp::http::StatusCode::OK))
    } else {
        let error_msg = json!({"success": false, "error": "Invalid signature"});
        let response = warp::reply::with_status(warp::reply::json(&error_msg), warp::http::StatusCode::BAD_REQUEST);
        Ok(response)
    }
}

fn generate_signature(channel_secret: &str, body: &[u8]) -> String {
    let mut hmac_sha256 = Hmac::<Sha256>::new_from_slice(channel_secret.as_bytes())
        .expect("Failed to create HMAC");
    hmac_sha256.update(&body);

    BASE64.encode(hmac_sha256.finalize().into_bytes())
}

fn with_channel_secret(
    channel_secret: Arc<Mutex<String>>,
) -> impl Filter<Extract = (Arc<Mutex<String>>,), Error = Infallible> + Clone {
    warp::any().map(move || channel_secret.clone())
}