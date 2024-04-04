extern crate dotenv;

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{routing::get, Router};
use dotenv::dotenv;
use itertools::Itertools;
use nanoid;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/images", post(post_image));
    let port = Env::default().PORT;
    let addr = format!("localhost:{port}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[allow(non_snake_case, dead_code)]
struct Env {
    PORT: u32,
    AWS_ACCESS_KEY: String,
    AWS_SECRET_ACCESS_KEY: String,
    AWS_S3_REGION: String,
    AWS_S3_BUCKET_NAME: String,
    AWS_CLOUDFRONT_DOMAIN: String,
}

impl Default for Env {
    fn default() -> Self {
        Env {
            PORT: env::var("PORT")
                .map(|v| v.parse().expect("Invalid environment variable PORT"))
                .unwrap_or(3001),
            AWS_ACCESS_KEY: env::var("PORT").expect("Missing environment variable AWS_ACCESS_KEY"),
            AWS_SECRET_ACCESS_KEY: env::var("AWS_SECRET_ACCESS_KEY")
                .expect("Missing environment variable AWS_SECRET_ACCESS_KEY"),
            AWS_S3_BUCKET_NAME: env::var("AWS_S3_BUCKET_NAME")
                .expect("Missing environment variable AWS_S3_BUCKET_NAME"),
            AWS_S3_REGION: env::var("AWS_S3_REGION")
                .expect("Missing environment variable AWS_S3_REGION"),
            AWS_CLOUDFRONT_DOMAIN: env::var("AWS_CLOUDFRONT_DOMAIN")
                .expect("Missing environment variable AWS_CLOUDFRONT_DOMAIN"),
        }
    }
}

async fn get_aws_sdk_client() -> Client {
    let env = Env::default();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(env.AWS_S3_REGION))
        .load()
        .await;
    Client::new(&config)
}

fn generate_id() -> String {
    let alphabet = "123456789abcdefghijklmnopqrstuvwxyz".chars().collect_vec();
    nanoid::nanoid!(12, &alphabet)
}

async fn post_image(mut multipart: Multipart) -> Result<String, StatusCode> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
    let client = get_aws_sdk_client().await;
    let env = Env::default();

    let entity_id = generate_id();
    let entity_key = format!("{entity_id}.jpeg");

    client
        .put_object()
        .bucket(env.AWS_S3_BUCKET_NAME)
        .key(&entity_key)
        .content_encoding("base64")
        .content_type("image/jpeg")
        .body(ByteStream::from(bytes))
        .send()
        .await
        .map(|_| {
            format!(
                "Image uploaded successfully, view it at https://{}/{}.",
                env.AWS_CLOUDFRONT_DOMAIN, entity_key
            )
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
