use std::env;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::json;
use tokio;

/**
 * Testing via the "Enums" approach.
 * https://docs.aws.amazon.com/sdk-for-rust/latest/dg/testing.html#testing-2
 */
enum PutEvents {
    Real(aws_sdk_eventbridge::Client),

    #[cfg(test)]
    Test {
        expected_detail: String,
        expected_source: String,
        expected_detail_type: String,
    },
}

impl PutEvents {
    async fn put_events(
        &self,
        entries: Vec<aws_sdk_eventbridge::model::PutEventsRequestEntry>,
    ) -> Result<
        aws_sdk_eventbridge::output::PutEventsOutput,
        aws_smithy_http::result::SdkError<aws_sdk_eventbridge::error::PutEventsError>,
    > {
        match self {
            /*
             * Because we operate in a trait, we use `Self` instead of `self` here.
             * The `Self` refers to the underlying object this trait is implemented on.
             * The `self` refers to the trait itself.
             */
            Self::Real(client) => Self::real_put_events(client, entries).await,
            #[cfg(test)]
            Self::Test {
                /*
                 * Why are these annotated as references??
                 */
                expected_detail,
                expected_detail_type,
                expected_source,
            } => {
                assert_eq!(entries.len(), 1);

                let entry = &entries[0];
                assert_eq!(entry.detail_type().unwrap(), expected_detail_type);
                assert_eq!(entry.source().unwrap(), expected_source);

                let detail_value: serde_json::Value =
                    serde_json::from_str(entry.detail().unwrap()).unwrap();
                let expected_detail_value: serde_json::Value =
                    serde_json::from_str(expected_detail).unwrap();

                assert_eq!(detail_value["id"].is_string(), true);
                assert_eq!(detail_value["message"], expected_detail_value["message"]);
                assert_eq!(detail_value["id"].is_string(), true);

                return Self::test_put_events(entries).await;
            }
        }
    }

    async fn real_put_events(
        client: &aws_sdk_eventbridge::Client,
        entries: Vec<aws_sdk_eventbridge::model::PutEventsRequestEntry>,
    ) -> Result<
        aws_sdk_eventbridge::output::PutEventsOutput,
        aws_smithy_http::result::SdkError<aws_sdk_eventbridge::error::PutEventsError>,
    > {
        return client.put_events().set_entries(Some(entries)).send().await;
    }

    #[cfg(test)]
    async fn test_put_events(
        entries: Vec<aws_sdk_eventbridge::model::PutEventsRequestEntry>,
    ) -> Result<
        aws_sdk_eventbridge::output::PutEventsOutput,
        aws_smithy_http::result::SdkError<aws_sdk_eventbridge::error::PutEventsError>,
    > {
        assert_eq!(entries.len(), 1);

        return Ok(aws_sdk_eventbridge::output::PutEventsOutput::builder().build());
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_eventbridge::Client::new(&config);

    let func = service_fn(|event| async {
        return handler(PutEvents::Real(client.clone()), event).await;
    });
    lambda_runtime::run(func).await?;

    return Ok(());
}

async fn handler(
    put_events_impl: PutEvents,
    event: LambdaEvent<aws_lambda_events::dynamodb::Event>,
) -> Result<(), Error> {
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

        let id = uuid::Uuid::new_v4().to_string();
        let detail = json!({"id": id, "message": message}).to_string();

        println!("Before putting event to EventBridge");

        let entry = aws_sdk_eventbridge::model::PutEventsRequestEntry::builder()
            .set_source(Some("dynamodb-function".to_string()))
            .set_detail(Some(detail))
            .set_detail_type(Some(detail_type))
            .set_event_bus_name(Some(event_bus_name))
            .build();

        put_events_impl.put_events(vec![entry]).await?;

        println!("After putting event to EventBridge")
    }

    return Ok(());
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn works_as_expected() -> Result<(), Error> {
        dotenv::dotenv().ok();

        let fake = PutEvents::Test {
            expected_detail: "{\"id\":\"123\",\"message\":\"dynamodb function says hello\"}"
                .to_string(),
            expected_source: "dynamodb-function".to_string(),
            expected_detail_type: "greeting".to_string(),
        };

        let event: aws_lambda_events::dynamodb::Event = serde_json::from_str(
            r#"{
                "Records": [
                    {
                        "eventID": "1",
                        "eventName": "INSERT",
                        "eventVersion": "1.1",
                        "eventSource": "aws:dynamodb",
                        "awsRegion": "us-west-2",
                        "dynamodb": {
                            "ApproximateCreationDateTime": 1582222222,
                            "Keys": {
                                "Id": {
                                    "N": "101"
                                }
                            },
                            "NewImage": {
                                "Message": {
                                    "S": "New item!"
                                },
                                "Id": {
                                    "N": "101"
                                }
                            },
                            "SequenceNumber": "111",
                            "SizeBytes": 26,
                            "StreamViewType": "NEW_AND_OLD_IMAGES"
                        },
                        "eventSourceARN": "arn:aws:dynamodb:us-west-2:account-id:table/ExampleTableWithStream/stream/2015-06-27T00:48:05.899"
                    }
                ]
            }"#,
        ).unwrap();
        handler(
            fake,
            lambda_runtime::LambdaEvent::new(event, lambda_runtime::Context::default()),
        )
        .await?;

        return Ok(());

        // handler(fake, event)
    }
}
