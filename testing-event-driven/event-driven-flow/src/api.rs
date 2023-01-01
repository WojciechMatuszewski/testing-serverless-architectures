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

async fn handler(
    _event: LambdaEvent<aws_lambda_events::apigw::ApiGatewayProxyRequest>,
) -> Result<aws_lambda_events::apigw::ApiGatewayProxyResponse, Error> {
    let event_bus_name = env::var("EVENT_BUS").expect("EVENT_BUS must be set");
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_eventbridge::Client::new(&config);

    let id = uuid::Uuid::new_v4().to_string();
    let detail = json!({"id": id, "message": "api function says hello"}).to_string();

    println!("Building EB entry");

    let entry = aws_sdk_eventbridge::model::PutEventsRequestEntry::builder()
        .set_source(Some("api-function".to_string()))
        .set_detail(Some(detail))
        .set_detail_type(Some("greeting".to_string()))
        .set_event_bus_name(Some(event_bus_name))
        .build();

    println!("Before putting event onto the bus");

    client.put_events().entries(entry).send().await?;

    println!("After putting event onto the bus");

    return Ok(aws_lambda_events::apigw::ApiGatewayProxyResponse {
        status_code: 200,
        body: None,
        ..Default::default()
    });
}
