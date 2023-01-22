use lambda_http::{run, service_fn, Body, Error, Request, Response, http::Uri};
use serde::Serialize;
use aws_sdk_dynamodb::{
    model::AttributeValue, Client,
};
use std::env;
use std::collections::HashMap;
use serde_json::json;
use tracing::info;
use std::time::Instant;

#[derive(Serialize, Clone)]
struct PackingListItem {
    id: String,
    check: bool,
    name: String,
    quantity: u32,
}

#[derive(Serialize, Clone)]
struct PackingList {
    id: String,
    name: String,
    items: Vec<PackingListItem>,
}

impl TryFrom<HashMap<String, AttributeValue>> for PackingListItem {
    type Error = Error;

    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(PackingListItem {
            check: value
                .get("check")
                .unwrap()
                .as_bool()
                .unwrap()
                .clone(),
            name: value
                .get("name")
                .unwrap()
                .as_s()
                .unwrap()
                .clone()
                .into(),
            quantity: value
                .get("quantity")
                .unwrap()
                .as_n()
                .unwrap()
                .parse()
                .unwrap(),
            id: value
                .get("id")
                .unwrap()
                .as_s()
                .unwrap()
                .parse()
                .unwrap(),
        })
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for PackingList {
    type Error = Error;

    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(PackingList {
            id: value
                .get("id")
                .ok_or(Error::from("id not found"))?
                .as_s()
                .unwrap()
                .clone(),
            name: value
                .get("name")
                .ok_or(Error::from("name not found"))?
                .as_s()
                .unwrap()
                .clone(),
            items: value
                .get("items")
                .ok_or(Error::from("items not found"))?
                .as_l()
                .unwrap()
                .iter()
                .map(|item| {
                    let item = item.as_m().unwrap();
                    PackingListItem::try_from(item.clone()).unwrap()
                })
                .collect(),
        })
    }
}

async fn get_packing_list(client: &Client, id: &str) -> Result<PackingList, Error> {
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

    match resp.item {
        Some(item) => {
            let packing_list = PackingList::try_from(item)?;
            Ok(packing_list)
        }
        None => Err(Error::from("Packing list not found by provided id")),
    }
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
        Ok(packing_list) => {
            let body = serde_json::to_string(&packing_list)?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(body))
                .unwrap())
        }
        Err(err) => {
            let error_body = json!({
                "error": err.to_string(),
            });

            Ok(Response::builder()
                .status(500)
                .body(Body::from(error_body.to_string())) 
                .unwrap())
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
