use std::env;

use async_trait::async_trait;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::json;
use tokio;

/**
 * Testing via the "Trait objects" approach
 * https://docs.aws.amazon.com/sdk-for-rust/latest/dg/testing.html#testing-1
 */

#[async_trait]
pub trait EventsPutter {
    async fn put_events(
        &self,
        entries: Option<Vec<aws_sdk_eventbridge::model::PutEventsRequestEntry>>,
    ) -> Result<
        aws_sdk_eventbridge::output::PutEventsOutput,
        aws_smithy_http::result::SdkError<aws_sdk_eventbridge::error::PutEventsError>,
    >;
}

#[derive(Clone, Debug)]
pub struct EventBridgePutEvents {
    client: aws_sdk_eventbridge::Client,
}

impl EventBridgePutEvents {
    pub fn new(client: aws_sdk_eventbridge::Client) -> Self {
        return Self { client };
    }
}

#[async_trait]
impl EventsPutter for EventBridgePutEvents {
    async fn put_events(
        &self,
        entries: Option<Vec<aws_sdk_eventbridge::model::PutEventsRequestEntry>>,
    ) -> Result<
        aws_sdk_eventbridge::output::PutEventsOutput,
        aws_smithy_http::result::SdkError<aws_sdk_eventbridge::error::PutEventsError>,
    > {
        return self.client.put_events().set_entries(entries).send().await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_eventbridge::Client::new(&config);
    let event_bridge_put_events = EventBridgePutEvents::new(client);

    let func = service_fn(|event| async {
        return handler(&event_bridge_put_events, event).await;
    });
    lambda_runtime::run(func).await?;

    return Ok(());
}

async fn handler(
    events_putter: &dyn EventsPutter,
    event: LambdaEvent<aws_lambda_events::apigw::ApiGatewayProxyRequest>,
) -> Result<aws_lambda_events::apigw::ApiGatewayProxyResponse, Error> {
    println!("Event: {:?}", event);

    let id = uuid::Uuid::new_v4().to_string();
    let detail = json!({"id": id, "message": "api function says hello"}).to_string();

    println!("Building EB entry");

    let event_bus_name = env::var("EVENT_BUS").expect("EVENT_BUS must be set");
    let entry = aws_sdk_eventbridge::model::PutEventsRequestEntry::builder()
        .set_source(Some("api-function".to_string()))
        .set_detail(Some(detail))
        .set_detail_type(Some("greeting".to_string()))
        .set_event_bus_name(Some(event_bus_name))
        .build();

    println!("Before putting event onto the bus");

    events_putter.put_events(Some(vec![entry])).await?;

    println!("After putting event onto the bus");

    return Ok(aws_lambda_events::apigw::ApiGatewayProxyResponse {
        status_code: 200,
        body: None,
        ..Default::default()
    });
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::EventsPutter;
    use async_trait::async_trait;
    use serde_json::json;

    #[tokio::test]
    async fn sends_correct_event() -> Result<(), Error> {
        let env_file_path =
            std::fs::canonicalize(PathBuf::from("./.env")).expect("Env file should exist");
        dotenv::from_path(env_file_path).unwrap();

        struct TestEventsPutter {
            expected_source: String,
            expected_detail: String,
            expected_detail_type: String,
        }

        #[async_trait]
        /**
         * Problem: assertions as implementation detail.
         * If this were to live in another file, the test would be hard to read.
         */
        impl EventsPutter for TestEventsPutter {
            async fn put_events(
                &self,
                entries: Option<Vec<aws_sdk_eventbridge::model::PutEventsRequestEntry>>,
            ) -> Result<
                aws_sdk_eventbridge::output::PutEventsOutput,
                aws_smithy_http::result::SdkError<aws_sdk_eventbridge::error::PutEventsError>,
            > {
                assert_eq!(entries.is_some(), true);
                let entries = entries.unwrap();
                assert_eq!(entries.len(), 1);

                let first_entry = entries.get(0).unwrap();

                let expected_detail_value: serde_json::Value =
                    serde_json::from_str(&self.expected_detail).unwrap();
                let entry_detail: serde_json::Value =
                    serde_json::from_str(first_entry.detail().unwrap()).unwrap();

                assert_eq!(entry_detail["message"], expected_detail_value["message"]);
                assert_eq!(entry_detail["id"].is_string(), true);

                assert_eq!(first_entry.source().unwrap(), self.expected_source);
                assert_eq!(
                    first_entry.detail_type().unwrap(),
                    self.expected_detail_type
                );
                return Ok(aws_sdk_eventbridge::output::PutEventsOutput::builder().build());
            }
        }

        let test_events_putter = TestEventsPutter {
            expected_source: "api-function".to_string(),
            /**
             * Problem: how to mock the id field here?
             */
            expected_detail: json!({"message": "api function says hello"}).to_string(),
            expected_detail_type: "greeting".to_string(),
        };

        let test_event = LambdaEvent::new(
            aws_lambda_events::apigw::ApiGatewayProxyRequest::default(),
            lambda_runtime::Context::default(),
        );

        handler(&test_events_putter, test_event).await?;
        return Ok(());
    }
}
