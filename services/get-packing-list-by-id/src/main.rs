use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::Serialize;
use aws_sdk_dynamodb::{
    model::AttributeValue, Client,
};
use std::env;

#[derive(Serialize, Clone)]
struct ResponseBody {
    name: String,
}


async fn function_handler(_: Request) -> Result<Response<Body>, Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    let item = client
        .get_item()
        .table_name(env::var("DYNAMODB_TABLE_NAME").unwrap())
        .key(
            "PK",
            AttributeValue::S("PACKING_LIST#123".to_string()),
        )
        .key(
            "SK",
            AttributeValue::S("PACKING_LIST#123".to_string()),
        )
        .send()
        .await?;

    let name = item.item.unwrap().get("name").unwrap().as_s().unwrap().clone().into();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/text")
        .body(name)
        .map_err(Box::new)?;

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
