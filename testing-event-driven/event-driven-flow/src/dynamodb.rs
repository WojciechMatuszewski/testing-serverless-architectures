use std::env;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::json;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;

    return Ok(());
}

async fn handler(event: LambdaEvent<aws_lambda_events::dynamodb::Event>) -> Result<(), Error> {
    for record in event.payload.records {
        println!("DynamoDB record: {:?}", record);

        let detail_type: String;
        let message: String;

        match record.event_name.as_str() {
            "INSERT" => {
                detail_type = "greeting".to_string();
                message = "dynamodb function says hello".to_string();
            }
            "MODIFY" => {
                detail_type = "greeting".to_string();
                message = "dynamodb function says hello, again".to_string();
            }
            "REMOVE" => {
                detail_type = "bye".to_string();
                message = "dynamodb function says good bye".to_string();
            }
            _ => {
                detail_type = "unknown".to_string();
                message = "unknown".to_string();
            }
        }

        println!("detail_type: {}, message: {}", detail_type, message);

        let event_bus_name = env::var("EVENT_BUS").expect("EVENT_BUS must be set");
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_eventbridge::Client::new(&config);

        let id = uuid::Uuid::new_v4().to_string();
        let detail = json!({"id": id, "message": message}).to_string();

        println!("Before putting event to EventBridge");

        let entry = aws_sdk_eventbridge::model::PutEventsRequestEntry::builder()
            .set_source(Some("dynamodb-function".to_string()))
            .set_detail(Some(detail))
            .set_detail_type(Some(detail_type))
            .set_event_bus_name(Some(event_bus_name))
            .build();
        client.put_events().entries(entry).send().await?;

        println!("After putting event to EventBridge")
    }

    return Ok(());
}
