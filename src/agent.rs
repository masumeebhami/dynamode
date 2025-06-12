use crate::error::{DynamodeError, Result};
use crate::model::DynamoModel;
use aws_sdk_dynamodb::config::Region;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde::{de::DeserializeOwned, Serialize};

pub struct DynamodeAgent {
    pub client: Client,
}

impl DynamodeAgent {
    /// Connects to DynamoDB running locally at http://localhost:8000
    pub async fn connect_local() -> Self {
        let region = Region::new("us-west-2");
        let shared_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        let dynamo_config = aws_sdk_dynamodb::config::Builder::from(&shared_config)
            .region(region) //
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
    /// Update an item (full overwrite)
    pub async fn update<M: DynamoModel + Serialize>(&self, item: &M) -> Result<()> {
        // In DynamoDB, put_item with the same key overwrites. For partial update, you would use UpdateItem.
        self.put(item).await
    }

    /// Delete an item by (pk, sk)
    pub async fn delete<M: DynamoModel>(&self, keys: (String, String)) -> Result<()> {
        let table_name = M::table_name();
        let (pk, sk) = keys;
        let mut key_map = std::collections::HashMap::new();
        key_map.insert("pk".to_string(), AttributeValue::S(pk));
        key_map.insert("sk".to_string(), AttributeValue::S(sk));

        self.client
            .delete_item()
            .table_name(table_name)
            .set_key(Some(key_map))
            .send()
            .await
            .map_err(|e| DynamodeError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    /// Query all items with a given partition key (e.g. all cars for "bmw")
    pub async fn query_by_pk<M: DynamoModel + DeserializeOwned>(
        &self,
        pk_value: String,
    ) -> Result<Vec<M>> {
        let table_name = M::table_name();
        let mut expr_attr_names = std::collections::HashMap::new();
        expr_attr_names.insert("#pk".to_string(), "pk".to_string());

        let mut expr_attr_vals = std::collections::HashMap::new();
        expr_attr_vals.insert(":pk_val".to_string(), AttributeValue::S(pk_value.clone()));

        let resp = self
            .client
            .query()
            .table_name(table_name)
            .key_condition_expression("#pk = :pk_val")
            .expression_attribute_names("#pk", "pk")
            .expression_attribute_values(":pk_val", AttributeValue::S(pk_value))
            .send()
            .await
            .map_err(|e| DynamodeError::DynamoDb(e.to_string()))?;

        let mut results = Vec::new();
        if let Some(items) = resp.items {
            for item in items {
                let mut map = serde_json::Map::new();
                for (k, v) in item {
                    map.insert(k, av_to_json_value(&v)?);
                }
                let json = serde_json::Value::Object(map);
                let model: M = serde_json::from_value(json)
                    .map_err(|e| DynamodeError::Deserialization(e.to_string()))?;
                results.push(model);
            }
        }
        Ok(results)
    }

    /// Scan all items in the table (admin/debug only!)
    pub async fn scan_all<M: DynamoModel + DeserializeOwned>(&self) -> Result<Vec<M>> {
        let table_name = M::table_name();

        let resp = self
            .client
            .scan()
            .table_name(table_name)
            .send()
            .await
            .map_err(|e| DynamodeError::DynamoDb(e.to_string()))?;

        let mut results = Vec::new();
        if let Some(items) = resp.items {
            for item in items {
                let mut map = serde_json::Map::new();
                for (k, v) in item {
                    map.insert(k, av_to_json_value(&v)?);
                }
                let json = serde_json::Value::Object(map);
                let model: M = serde_json::from_value(json)
                    .map_err(|e| DynamodeError::Deserialization(e.to_string()))?;
                results.push(model);
            }
        }
        Ok(results)
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
