use lambda_http::{run, service_fn, Body, Error, Request, Response};
use aws_sdk_dynamodb::{
    model::AttributeValue, Client,
};
use std::env;
use model::PackingList;
use uuid::Uuid;


async fn put_packing_list(client: &Client, packing_list: PackingList) -> Result<(), Error> {
    let id = Uuid::new_v4();

    let pk = format!("{}{}", "PACKING_LIST#", id);
    let sk = format!("{}{}", "PACKING_LIST#", id);

    let resp = client.put_item()
        .table_name(env::var("DYNAMODB_TABLE_NAME").unwrap())
        .item("PK", AttributeValue::S(pk))
        .item("SK", AttributeValue::S(sk))
        .item("id", AttributeValue::S(packing_list.id))
        .item("name", AttributeValue::S(packing_list.name))
        .send()
        .await?;
} 

async fn function_handler(client: &Client, request: Request) -> Result<Response<Body>, Error> {
    info!("Request: {:?}", request);
    let body = request.body();
    let packing_list: PackingList = serde_json::from_slice(body)?;
    
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
