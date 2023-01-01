use std::{collections::HashMap, env};

use lambda_runtime::{service_fn, Error, LambdaEvent};

use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;

    return Ok(());
}

async fn handler(event: LambdaEvent<aws_lambda_events::sns::SnsEvent>) -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    for record in event.payload.records {
        println!("SNS record: {:?}", record);

        let message = record.sns.message;
        let id = record
            .sns
            .message_attributes
            .get("id")
            .unwrap()
            .value
            .clone();

        let mut get_item_key: HashMap<String, aws_sdk_dynamodb::model::AttributeValue> =
            HashMap::new();
        get_item_key.insert(
            "id".to_string(),
            aws_sdk_dynamodb::model::AttributeValue::S(id.clone()),
        );

        println!("Before get item response");

        let get_response = dynamodb_client
            .get_item()
            .set_table_name(Some(
                env::var("DYNAMODB_TABLE").expect("TABLE_NAME must be set"),
            ))
            .set_key(Some(get_item_key))
            .send()
            .await?;

        println!("After get item response");

        if get_response.item().is_none() {
            println!("Item not found");
            return Ok(());
        }

        let mut put_item_key: HashMap<String, aws_sdk_dynamodb::model::AttributeValue> =
            HashMap::new();
        put_item_key.insert(
            "id".to_string(),
            aws_sdk_dynamodb::model::AttributeValue::S(id),
        );

        println!("Before item update");

        dynamodb_client
            .update_item()
            .set_table_name(Some(
                env::var("DYNAMODB_TABLE").expect("TABLE_NAME must be set"),
            ))
            .set_key(Some(put_item_key))
            .set_update_expression(Some("SET #ending = :ending".to_string()))
            .expression_attribute_names("#ending", "ending")
            .expression_attribute_values(
                ":ending",
                aws_sdk_dynamodb::model::AttributeValue::S(format!(
                    "{}, and the sns function says good bye",
                    message
                )),
            )
            .send()
            .await?;

        println!("After item update");
    }

    return Ok(());
}
