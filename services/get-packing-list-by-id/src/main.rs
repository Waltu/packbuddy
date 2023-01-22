use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::Serialize;
use aws_sdk_dynamodb::{
    model::AttributeValue, Client,
};
use std::env;
use std::collections::HashMap;

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

impl TryFrom<HashMap<String, AttributeValue>> for PackingList {
    type Error = Error;

    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(PackingList {
            id: value
                .get("id")
                .unwrap()
                .as_s()
                .unwrap()
                .clone()
                .into(),
            name: value
                .get("name")
                .unwrap()
                .as_s()
                .unwrap()
                .clone()
                .into(),
            items: value
                .get("items")
                .unwrap()
                .as_l()
                .unwrap()
                .iter()
                .map(|item| {
                    let item = item.as_m().unwrap();
                    PackingListItem {
                        check: item
                            .get("check")
                            .unwrap()
                            .as_bool()
                            .unwrap()
                            .clone(),
                        name: item
                            .get("name")
                            .unwrap()
                            .as_s()
                            .unwrap()
                            .clone()
                            .into(),
                        quantity: item
                            .get("quantity")
                            .unwrap()
                            .as_n()
                            .unwrap()
                            .parse()
                            .unwrap(),
                        id: item
                            .get("id")
                            .unwrap()
                            .as_n()
                            .unwrap()
                            .parse()
                            .unwrap(),

                    }
                })
                .collect(),
        })
    }
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

    Ok(PackingList::try_from(item).unwrap())
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
