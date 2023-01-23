use lambda_http::{run, service_fn, Body, Error, Request, Response, http::Uri, http::StatusCode};
use aws_sdk_dynamodb::{
    model::AttributeValue, Client,
};
use std::env;
use serde_json::json;
use tracing::{info, error};
use std::time::Instant;
use model::PackingList;


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

fn response(status_code: StatusCode, body: String) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

async fn function_handler(client: &Client, request: Request) -> Result<Response<Body>, Error> {
    info!("Request: {:?}", request);
    let packing_list_id = get_id_from_uri(request.uri())?;

    let packing_list = get_packing_list(client, packing_list_id).await;

    match packing_list {
        Ok(Some(packing_list)) => {
            let body = serde_json::to_string(&packing_list)?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(body))
                .unwrap())
        },
        Ok(None) => Ok(response(StatusCode::NOT_FOUND, json!({"message": "packing list not found"}).to_string())),
        Err(err) => {
            error!("Error fetching packing list: {}", err);

            let error_body = json!({
                "error": err.to_string(),
            });

            Ok(response(StatusCode::INTERNAL_SERVER_ERROR, error_body.to_string()))
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
