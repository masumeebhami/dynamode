# Dynamode

A professional, ergonomic, and extensible Rust Object-Document Mapper (ODM) for DynamoDB  
Supports **DynamoDB Local** out-of-the-box for fast local development and testing.

---

## 🚀 Features

- Async Rust API for DynamoDB (powered by [aws-sdk-dynamodb](https://crates.io/crates/aws-sdk-dynamodb))
- Define models as Rust structs with familiar traits
- Works with DynamoDB Local for easy testing/development
- Table auto-creation helpers
- CRUD operations: insert, fetch, update, delete
- Query by partition key, full-table scan, and more!
- Ready for expansion: batch ops, GSIs, conditional writes, etc.

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dynamode = { version = last }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
aws-sdk-dynamodb = "1"
aws-config = "1"
async-trait = "0.1"
```

## 🛠️ Running DynamoDB Local
With Docker (recommended):
```
docker run -d -p 8000:8000 amazon/dynamodb-local
```
Set dummy AWS credentials (required by the SDK):
```
export AWS_ACCESS_KEY_ID=dummy
export AWS_SECRET_ACCESS_KEY=dummy
```
🚗 Quick Example
```
use serde::{Serialize, Deserialize};
use dynamode::agent::DynamodeAgent;
use dynamode::model::DynamoModel;

#[derive(Debug, Serialize, Deserialize)]
struct Car {
    pk: String,
    sk: String,
    brand: String,
    model: String,
    horsepower: i32,
}

#[async_trait::async_trait]
impl DynamoModel for Car {
    fn table_name() -> &'static str { "Cars" }
    fn partition_sort_key(&self) -> (String, String) {
        (self.pk.clone(), self.sk.clone())
    }
}

#[tokio::main]
async fn main() {
    let agent = DynamodeAgent::connect_local().await;

    let car = Car {
        pk: "tesla".into(),
        sk: "model-y".into(),
        brand: "Tesla".into(),
        model: "Model Y".into(),
        horsepower: 420,
    };

    agent.put(&car).await.expect("insert failed");
    let fetched = agent.get::<Car>(("tesla".into(), "model-y".into())).await.expect("get failed");
    println!("Fetched: {:?}", fetched);
}
```
## 📋 Test Example
All features work with cargo test using DynamoDB Local!
Test cases are in main.rs as async functions.

## 🏗️ Table Creation
The first run will auto-create your table if it does not exist (see main.rs example).
You can also create tables manually via the AWS CLI:

```
aws dynamodb create-table \
    --table-name Cars \
    --attribute-definitions AttributeName=pk,AttributeType=S AttributeName=sk,AttributeType=S \
    --key-schema AttributeName=pk,KeyType=HASH AttributeName=sk,KeyType=RANGE \
    --billing-mode PAY_PER_REQUEST \
    --endpoint-url http://localhost:8000 \
    --region us-west-2
```
## ⚡ Tips
Always run DynamoDB Local (docker run -p 8000:8000 amazon/dynamodb-local) before developing or running tests.

Set AWS credentials in your shell session, even for local.

You can extend the agent for batch ops, transactional writes, secondary indexes, and more.

## 🧑‍💻 Contributing
PRs, issues, and feature requests are welcome!
Feel free to fork and build your own supercharged DynamoDB ORM.

## 📝 License
MIT

## 📣 Credits
Built by Rust and DynamoDB fans, with inspiration from the AWS SDK team and open source community.
