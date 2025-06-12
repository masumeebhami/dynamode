use aws_sdk_dynamodb::types::{
    AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType,
};
use dynamode::agent::DynamodeAgent;
use dynamode::model::DynamoModel;
use serde_derive::{Deserialize, Serialize};

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
    fn table_name() -> &'static str {
        "Cars"
    }
    fn partition_sort_key(&self) -> (String, String) {
        (self.pk.clone(), self.sk.clone())
    }
}

#[tokio::main]
async fn main() {
    // Connect to local DynamoDB
    let agent = DynamodeAgent::connect_local().await;
    let client = &agent.client;

    // 1. Create the Cars table if it does not exist
    let tables = client.list_tables().send().await.unwrap();
    if !tables.table_names().contains(&"Cars".to_string()) {
        println!("Creating Cars table...");
        client
            .create_table()
            .table_name("Cars")
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("pk")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .unwrap(),
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("sk")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .unwrap(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("pk")
                    .key_type(KeyType::Hash)
                    .build()
                    .unwrap(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("sk")
                    .key_type(KeyType::Range)
                    .build()
                    .unwrap(),
            )
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await
            .unwrap();

        println!("Cars table created!");
    } else {
        println!("Cars table already exists.");
    }

    // 2. Insert a car
    let car = Car {
        pk: "tesla".into(),
        sk: "model-y".into(),
        brand: "Tesla".into(),
        model: "Model Y".into(),
        horsepower: 420,
    };

    match agent.put(&car).await {
        Ok(_) => println!("Car inserted."),
        Err(e) => eprintln!("Insert failed: {}", e),
    }

    // 3. Fetch the car
    match agent.get::<Car>(("tesla".into(), "model-y".into())).await {
        Ok(Some(car)) => println!("Found car: {:?}", car),
        Ok(None) => println!("No car found."),
        Err(e) => eprintln!("Fetch failed: {}", e),
    }

    // 4. List tables again
    let tables = client.list_tables().send().await.unwrap();
    println!("Current tables: {:?}", tables.table_names());
}
