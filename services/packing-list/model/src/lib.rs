use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::Error;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub struct PackingListItem {
    id: String,
    check: bool,
    name: String,
    quantity: u32,
}

#[derive(Serialize, Clone)]
pub struct PackingList {
    id: String,
    name: String,
    items: Vec<PackingListItem>,
}

impl TryFrom<HashMap<String, AttributeValue>> for PackingListItem {
    type Error = Error;

    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(PackingListItem {
            check: value.get("check").unwrap().as_bool().unwrap().clone(),
            name: value.get("name").unwrap().as_s().unwrap().clone().into(),
            quantity: value
                .get("quantity")
                .unwrap()
                .as_n()
                .unwrap()
                .parse()
                .unwrap(),
            id: value.get("id").unwrap().as_s().unwrap().parse().unwrap(),
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
