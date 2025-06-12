use crate::error::{DynamodeError, Result};
use crate::model::DynamoModel;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client};
use serde::{de::DeserializeOwned, Serialize};

pub struct DynamodeAgent {
    pub client: Client,
}

impl DynamodeAgent {
    /// Connects to DynamoDB running locally at http://localhost:8000
    pub async fn connect_local() -> Self {
        let shared_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        // Use endpoint_url (no Endpoint struct needed)
        let dynamo_config = aws_sdk_dynamodb::config::Builder::from(&shared_config)
            .endpoint_url("http://localhost:8000")
            .build();

        let client = Client::from_conf(dynamo_config);
        Self { client }
    }

    /// Put item into DynamoDB
    pub async fn put<M: DynamoModel + Serialize>(&self, item: &M) -> Result<()> {
        let table_name = M::table_name();
        let item_json =
            serde_json::to_value(item).map_err(|e| DynamodeError::Serialization(e.to_string()))?;
        let mut item_map = std::collections::HashMap::new();

        if let serde_json::Value::Object(map) = item_json {
            for (k, v) in map {
                let av = json_value_to_av(v)?;
                item_map.insert(k, av);
            }
        }

        self.client
            .put_item()
            .table_name(table_name)
            .set_item(Some(item_map))
            .send()
            .await
            .map_err(|e| DynamodeError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    /// Get item by (pk, sk)
    pub async fn get<M: DynamoModel + DeserializeOwned>(
        &self,
        keys: (String, String),
    ) -> Result<Option<M>> {
        let table_name = M::table_name();
        let (pk, sk) = keys;
        let mut key_map = std::collections::HashMap::new();
        key_map.insert("pk".to_string(), AttributeValue::S(pk));
        key_map.insert("sk".to_string(), AttributeValue::S(sk));

        let output = self
            .client
            .get_item()
            .table_name(table_name)
            .set_key(Some(key_map))
            .send()
            .await
            .map_err(|e| DynamodeError::DynamoDb(e.to_string()))?;

        if let Some(item) = output.item {
            // Convert HashMap<String, AttributeValue> to serde_json::Value
            let mut map = serde_json::Map::new();
            for (k, v) in item {
                map.insert(k, av_to_json_value(&v)?);
            }
            let json = serde_json::Value::Object(map);
            let model: M = serde_json::from_value(json)
                .map_err(|e| DynamodeError::Deserialization(e.to_string()))?;
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}

// Helper: Convert serde_json::Value to AttributeValue
fn json_value_to_av(value: serde_json::Value) -> Result<AttributeValue> {
    match value {
        serde_json::Value::String(s) => Ok(AttributeValue::S(s)),
        serde_json::Value::Number(num) => {
            if let Some(n) = num.as_i64() {
                Ok(AttributeValue::N(n.to_string()))
            } else if let Some(n) = num.as_u64() {
                Ok(AttributeValue::N(n.to_string()))
            } else if let Some(n) = num.as_f64() {
                Ok(AttributeValue::N(n.to_string()))
            } else {
                Err(DynamodeError::Serialization(
                    "Number parse error".to_string(),
                ))
            }
        }
        serde_json::Value::Bool(b) => Ok(AttributeValue::Bool(b)),
        serde_json::Value::Null => Ok(AttributeValue::Null(true)),
        serde_json::Value::Array(arr) => {
            let mut vals = vec![];
            for v in arr {
                vals.push(json_value_to_av(v)?);
            }
            Ok(AttributeValue::L(vals))
        }
        serde_json::Value::Object(map) => {
            let mut av_map = std::collections::HashMap::new();
            for (k, v) in map {
                av_map.insert(k, json_value_to_av(v)?);
            }
            Ok(AttributeValue::M(av_map))
        }
    }
}

// Helper: Convert AttributeValue to serde_json::Value
fn av_to_json_value(av: &AttributeValue) -> Result<serde_json::Value> {
    match av {
        AttributeValue::S(s) => Ok(serde_json::Value::String(s.clone())),
        AttributeValue::N(n) => {
            if let Ok(i) = n.parse::<i64>() {
                Ok(serde_json::Value::Number(i.into()))
            } else if let Ok(f) = n.parse::<f64>() {
                Ok(serde_json::Value::Number(
                    serde_json::Number::from_f64(f).unwrap(),
                ))
            } else {
                Ok(serde_json::Value::String(n.clone()))
            }
        }
        AttributeValue::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        AttributeValue::Null(_) => Ok(serde_json::Value::Null),
        AttributeValue::L(lst) => {
            let mut vals = vec![];
            for v in lst {
                vals.push(av_to_json_value(v)?);
            }
            Ok(serde_json::Value::Array(vals))
        }
        AttributeValue::M(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                json_map.insert(k.clone(), av_to_json_value(v)?);
            }
            Ok(serde_json::Value::Object(json_map))
        }
        _ => Err(DynamodeError::Deserialization(
            "Unsupported AttributeValue".to_string(),
        )),
    }
}
