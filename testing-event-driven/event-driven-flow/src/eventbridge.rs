use std::{collections::HashMap, env};

use lambda_runtime::{service_fn, Error, LambdaEvent};

use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;

    return Ok(());
}

async fn handler(
    event: LambdaEvent<aws_lambda_events::cloudwatch_events::CloudWatchEvent>,
) -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    println!("Event payload: {:?}", event.payload.clone());

    let table_name = env::var("DYNAMODB_TABLE").expect("TABLE_NAME must be set");
    let event_source = event.payload.source.expect("Source must be there");

    let event_detail = event.payload.detail.expect("Detail must be present");
    let event_id = event_detail["id"].to_string();
    let event_message = event_detail["message"].to_string();

    println!("Creating DynamoDB item");

    let mut item: HashMap<String, aws_sdk_dynamodb::model::AttributeValue> = HashMap::new();
    item.insert(
        "id".to_string(),
        aws_sdk_dynamodb::model::AttributeValue::S(event_id.clone()),
    );
    item.insert(
        "source".to_string(),
        aws_sdk_dynamodb::model::AttributeValue::S(event_source),
    );
    item.insert(
        "message".to_string(),
        aws_sdk_dynamodb::model::AttributeValue::S(event_message),
    );

    println!("DynamoDB item created");

    println!("Before putting DynamoDB item");

    dynamodb_client
        .put_item()
        .set_table_name(Some(table_name))
        .set_item(Some(item))
        .send()
        .await?;

    println!("After putting DynamoDB item");

    let topic_arn = env::var("SNS_TOPIC_ARN").expect("SNS_TOPIC_ARN must be set");
    let sns_client = aws_sdk_sns::Client::new(&config);

    println!("Creating SNS message");

    let mut message_attributes: HashMap<String, aws_sdk_sns::model::MessageAttributeValue> =
        HashMap::new();
    message_attributes.insert(
        "id".to_string(),
        aws_sdk_sns::model::MessageAttributeValue::builder()
            .set_data_type(Some("String".to_string()))
            .set_string_value(Some(event_id))
            .build(),
    );

    println!("Created SNS message: {:?}", message_attributes);

    println!("Before publishing SNS message");

    sns_client
        .publish()
        .set_topic_arn(Some(topic_arn))
        .set_message(Some("eventbridge function says hello".to_string()))
        .set_message_attributes(Some(message_attributes))
        .send()
        .await?;

    println!("After publishing SNS message");

    return Ok(());
}
