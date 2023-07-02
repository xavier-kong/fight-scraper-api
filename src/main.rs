use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};
use planetscale_driver::{query, Database, PSConnection};
use std::env::var;
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};
use std::env;

//// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples

#[derive(Debug, Deserialize)]
struct MyRequest {
    offset: String,
}

#[derive(Database, Debug)]
pub struct Event {
    id: i32,
    name: String,
    timestamp_seconds: i32,
    headline: String,
    url: String,
    org: String
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let request_body: MyRequest = serde_json::from_slice(event.body().as_ref())?;

    let offset_int: i8 = request_body.offset.parse().unwrap_or(0);

    let url = env::var("DATABASE_URL").expect("DATABASE_URL not found");
    let builder = mysql::OptsBuilder::from_opts(mysql::Opts::from_url(&url).unwrap());
    let pool = mysql::Pool::new(builder.ssl_opts(mysql::SslOpts::default())).unwrap();
    let conn = pool.get_conn().unwrap();
    println!("Successfully connected to PlanetScale!");


    let curr_time_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let query_string = std::format!("SELECT * FROM events WHERE timestamp_seconds > {}", curr_time_secs);

    let query_res = conn.query(query_string);

    let message = "hello";

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
