# Line Messaging API Rust Example

This is an example Rust project that uses the LINE Messaging API to verify and parse webhook events. It utilizes the `warp` web framework to handle HTTP requests.

## Project Structure

- `main.rs`: The main entry point of the application. It sets up the `warp` routes and runs the web server.
- `mod.rs`: Contains utility functions that are used in `main.rs`.
- `config.toml`: Contains configuration settings for the Line Messaging API.

## Dependencies

- `config`: A configuration file management library.
- `serde_json`: A JSON serialization and deserialization library.
- `warp`: A web framework for Rust.
- `base64`: A library for encoding and decoding base64 data.
- `bytes`: A library for working with byte sequences.
- `hmac`: A library for creating and verifying HMACs.
- `sha2`: A library for computing SHA-2 hash functions.

## Functionality

The application sets up two endpoints:

1. `POST /verify`: Verifies the request signature and parses the webhook event. If the signature is valid, returns a 200 OK response. Otherwise, returns a 400 Bad Request response.
2. `GET /hello`: Returns a JSON response with the message `"success": true`.

The `verify` endpoint takes in the following parameters:

- `x-line-signature`: The signature of the request.
- `body`: The body of the request.

The `parse_request_handler` function in `mod.rs` is responsible for verifying the signature and parsing the webhook event.

The `with_channel_secret` function in `mod.rs` is used to pass the `channel_secret` value to the `parse_request_handler` function using the `Arc` and `Mutex` constructs to handle concurrency.
