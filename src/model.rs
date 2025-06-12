use async_trait::async_trait;

/// Trait for any struct to be used as a DynamoDB entity.
#[async_trait]
pub trait DynamoModel: Send + Sync {
    fn table_name() -> &'static str
    where
        Self: Sized;
    fn partition_sort_key(&self) -> (String, String);
}
