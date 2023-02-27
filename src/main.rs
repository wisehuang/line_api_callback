use config::{Config, File, FileFormat};
use serde_json::json;
use std::sync::{Arc, Mutex};
use warp::Filter;

mod r#mod;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    let config_builder = Config::builder().add_source(File::new("config.toml", FileFormat::Toml));

    let channel_secret = r#mod::get_channel_secret(config_builder);

    let parse_request_route = warp::post()
        .and(warp::path("verify"))
        .and(warp::header::<String>("x-line-signature"))
        .and(warp::body::bytes())
        .and(r#mod::with_channel_secret(Arc::new(Mutex::new(
            channel_secret,
        ))))
        .and_then(r#mod::parse_request_handler);

    let test_route = warp::get()
        .and(warp::path("hello"))
        .map(|| Ok(warp::reply::json(&json!({"success": true}))));

    let log_filter = warp::log("line_api_callback");

    let routes = parse_request_route.or(test_route).with(log_filter);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
