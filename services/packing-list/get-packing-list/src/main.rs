use lambda_http::{run, service_fn, Body, Error, Request, Response, http::Uri, http::StatusCode};
use aws_sdk_dynamodb::{
    model::AttributeValue, Client,
};
use std::env;
use serde_json::json;
use tracing::{info, error, warn};
use std::time::Instant;
use model::PackingList;
use utils::response;


async fn get_packing_list(client: &Client, id: &str) -> Result<Option<PackingList>, Error> {
    let pk = format!("{}{}", "PACKING_LIST#", id);
    let sk = format!("{}{}", "PACKING_LIST#", id);

    let start = Instant::now();

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

    let duration = start.elapsed().as_millis();
    info!("Time elapsed get_item function: {:?}ms", duration);

    Ok(match resp.item {
        Some(item) => {
            let packing_list = PackingList::try_from(item)?;
            Some(packing_list)
        }
        None => None,
    })
} 

fn get_id_from_uri(uri: &Uri) -> Result<&str, Error> {
    let id = uri.path().split("/").last();

    match id {
        Some(id) => Ok(id),
        None => Err(Error::from("id not found from path")),
    }
}

async fn function_handler(client: &Client, request: Request) -> Result<Response<Body>, Error> {
    info!("Request: {:?}", request);
    let packing_list_id = get_id_from_uri(request.uri())?;

    let packing_list = get_packing_list(client, packing_list_id).await;

    match packing_list {
        Ok(Some(packing_list)) => {
            let body = serde_json::to_string(&packing_list)?;
            Ok(response(StatusCode::OK, body))
        },
        Ok(None) => {
            warn!("Packing list not found by ID: {}", packing_list_id);
            Ok(response(StatusCode::NOT_FOUND, json!({"message": "packing list not found"}).to_string()))
        },
        Err(err) => {
            error!("Error fetching packing list: {}", err);

            Ok(response(StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": err.to_string() }).to_string()))
        }
    }
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
