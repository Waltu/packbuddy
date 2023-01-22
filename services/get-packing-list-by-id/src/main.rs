use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::Serialize;
use aws_sdk_dynamodb::{
    model::AttributeValue, Client,
};
use std::env;

#[derive(Serialize, Clone)]
struct PackingList {
    id: String,
    name: String,
}

async fn get_packing_list(client: &Client, id: &str) -> Result<PackingList, Error> {
    let pk = format!("{}{}", "PACKING_LIST#", id);
    let sk = format!("{}{}", "PACKING_LIST#", id);

    let resp = client.get_item()
        .table_name(env::var("DYNAMODB_TABLE_NAME").unwrap())
        .key(
            "PK",
            AttributeValue::S(pk),
        )
        .key(
            "SK",
            AttributeValue::S(sk),
        )
        .send()
        .await?;

    let item = resp.item.unwrap();

    Ok(PackingList {
        id: item
            .get("id")
            .unwrap()
            .as_s()
            .unwrap()
            .clone()
            .into(),
        name: item
            .get("name")
            .unwrap()
            .as_s()
            .unwrap()
            .clone()
            .into(),
    })
} 

async fn function_handler(client: &Client, _: Request) -> Result<Response<Body>, Error> {
    let packing_list = get_packing_list(client, "123").await?;

    let serialized_packing_list = serde_json::to_string(&packing_list).unwrap();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serialized_packing_list.into())
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


    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    run(service_fn(|event: Request| function_handler(&client, event))).await?;
    Ok(())
}
