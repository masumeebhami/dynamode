use aws_sdk_dynamodb::types::{
    AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType,
};
use dynamode::agent::DynamodeAgent;
use dynamode::model::DynamoModel;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    // Setup
    let agent = DynamodeAgent::connect_local().await;
    let client = &agent.client;

    // Create table if needed
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

    // Insert and fetch for demo
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

    match agent.get::<Car>(("tesla".into(), "model-y".into())).await {
        Ok(Some(car)) => println!("Found car: {:?}", car),
        Ok(None) => println!("No car found."),
        Err(e) => eprintln!("Fetch failed: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_insert_and_get_car() {
        let agent = DynamodeAgent::connect_local().await;

        let car = Car {
            pk: "bmw".into(),
            sk: "m3".into(),
            brand: "BMW".into(),
            model: "M3".into(),
            horsepower: 473,
        };

        agent.put(&car).await.expect("Insert failed");
        let fetched = agent
            .get::<Car>(("bmw".into(), "m3".into()))
            .await
            .expect("Get failed");
        assert_eq!(fetched, Some(car));
    }

    #[tokio::test]
    async fn test_update_car() {
        let agent = DynamodeAgent::connect_local().await;

        let mut car = Car {
            pk: "bmw".into(),
            sk: "m3".into(),
            brand: "BMW".into(),
            model: "M3".into(),
            horsepower: 473,
        };
        agent.put(&car).await.expect("Insert failed");

        // Update horsepower
        car.horsepower = 503;
        agent.update(&car).await.expect("Update failed");

        let fetched = agent
            .get::<Car>(("bmw".into(), "m3".into()))
            .await
            .expect("Get failed");
        assert_eq!(fetched.unwrap().horsepower, 503);
    }

    #[tokio::test]
    async fn test_delete_car() {
        let agent = DynamodeAgent::connect_local().await;
        let car = Car {
            pk: "toyota".into(),
            sk: "supra".into(),
            brand: "Toyota".into(),
            model: "Supra".into(),
            horsepower: 335,
        };
        agent.put(&car).await.expect("Insert failed");

        agent
            .delete::<Car>(("toyota".into(), "supra".into()))
            .await
            .expect("Delete failed");

        let fetched = agent
            .get::<Car>(("toyota".into(), "supra".into()))
            .await
            .expect("Get failed");
        assert_eq!(fetched, None);
    }

    #[tokio::test]
    async fn test_query_by_pk() {
        let agent = DynamodeAgent::connect_local().await;
        let car1 = Car {
            pk: "audi".into(),
            sk: "rs7".into(),
            brand: "Audi".into(),
            model: "RS7".into(),
            horsepower: 591,
        };
        let car2 = Car {
            pk: "audi".into(),
            sk: "a4".into(),
            brand: "Audi".into(),
            model: "A4".into(),
            horsepower: 201,
        };
        agent.put(&car1).await.expect("Insert failed");
        agent.put(&car2).await.expect("Insert failed");

        let results = agent
            .query_by_pk::<Car>("audi".into())
            .await
            .expect("Query failed");
        assert!(results.iter().any(|c| c.sk == "rs7"));
        assert!(results.iter().any(|c| c.sk == "a4"));
    }

    #[tokio::test]
    async fn test_scan_all() {
        let agent = DynamodeAgent::connect_local().await;

        let results = agent.scan_all::<Car>().await.expect("Scan failed");
        // This just checks that scan returns without error and gives a vector
        println!("Scan returned {} items.", results.len());
    }
}
