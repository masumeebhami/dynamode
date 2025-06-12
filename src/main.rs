use dynamode::agent::DynamodeAgent;
use dynamode::model::DynamoModel;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Car {
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
        (self.brand.clone(), self.model.clone())
    }
}

#[tokio::main]
async fn main() {
    let agent = DynamodeAgent::connect_local().await;
    let car = Car {
        brand: "Toyota".into(),
        model: "Corolla".into(),
        horsepower: 140,
    };

    match agent.put(&car).await {
        Ok(_) => println!("Car inserted."),
        Err(e) => eprintln!("Insert failed: {}", e),
    }

    match agent.get::<Car>(("Toyota".into(), "Corolla".into())).await {
        Ok(Some(car)) => println!("Found car: {:?}", car),
        Ok(None) => println!("No car found."),
        Err(e) => eprintln!("Fetch failed: {}", e),
    }
}
