use lambda_runtime::{service_fn, LambdaEvent};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;

    return Ok(());
}

#[derive(Debug, Serialize, Deserialize)]
struct Input {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    pub url: String,
    pub size: usize,
}

async fn handler(event: LambdaEvent<Input>) -> Result<Output, lambda_runtime::Error> {
    let bytes = reqwest::get(&event.payload.url).await?.bytes().await?;
    let output = Output {
        url: event.payload.url,
        size: bytes.len(),
    };

    return Ok(output);
}

#[cfg(test)]
mod tests {
    use std::env;

    use aws_sdk_sfn::Region;
    use tokio_retry::{strategy, Retry};

    use crate::assert_machine_status;

    #[tokio::test]
    async fn tests_the_machine_via_sfn_local() {
        dotenv::from_filename(".env-outputs")
            .ok()
            .expect("Failed to load .env-outputs");

        let config = aws_config::load_from_env().await;
        let sfn_config = aws_sdk_sfn::config::Builder::from(&config)
            .region(Region::new(env::var("AwsRegion").unwrap()))
            .build();
        let sfn_client = aws_sdk_sfn::Client::from_conf(sfn_config);

        let local_sfn_config = aws_sdk_sfn::config::Builder::from(&config)
            .endpoint_resolver(
                aws_sdk_sfn::Endpoint::immutable("http://localhost:8083")
                    .expect("Invalid endpoint"),
            )
            .region(Region::new(env::var("AwsRegion").unwrap()))
            .build();
        let local_sfn_client = aws_sdk_sfn::Client::from_conf(local_sfn_config);

        let current_state_machine = sfn_client
            .describe_state_machine()
            .state_machine_arn(env::var("StateMachineArn").unwrap())
            .send()
            .await
            .unwrap();

        let create_local_state_machine_response = local_sfn_client
            .create_state_machine()
            .definition(current_state_machine.definition.unwrap())
            .name("SimpleExample")
            .role_arn(current_state_machine.role_arn.unwrap())
            .send()
            .await
            .unwrap();

        let is_big_html_arn = format!(
            "{}#IsBigPath",
            create_local_state_machine_response
                .state_machine_arn()
                .unwrap()
        );
        let start_is_big_html_response = local_sfn_client
            .start_execution()
            .state_machine_arn(is_big_html_arn)
            .input(r#"{"url": "https://www.rust-lang.org/"}"#)
            .send()
            .await
            .unwrap();

        let is_big_html_result =
            Retry::spawn(strategy::FixedInterval::from_millis(500).take(4), || {
                assert_machine_status(
                    &local_sfn_client,
                    start_is_big_html_response.execution_arn().unwrap(),
                    aws_sdk_sfn::model::ExecutionStatus::Succeeded,
                )
            })
            .await;

        assert_eq!(is_big_html_result.unwrap(), "true".to_string());

        let is_small_html_arn = format!(
            "{}#IsNotBigPath",
            create_local_state_machine_response
                .state_machine_arn()
                .unwrap()
        );
        let start_is_small_html_response = local_sfn_client
            .start_execution()
            .state_machine_arn(is_small_html_arn)
            .input(r#"{"url": "https://www.rust-lang.org/"}"#)
            .send()
            .await
            .unwrap();

        let is_small_html_result =
            Retry::spawn(strategy::FixedInterval::from_millis(500).take(4), || {
                assert_machine_status(
                    &local_sfn_client,
                    start_is_small_html_response.execution_arn().unwrap(),
                    aws_sdk_sfn::model::ExecutionStatus::Succeeded,
                )
            })
            .await;
        assert_eq!(is_small_html_result.unwrap(), "false".to_string());

        let is_error_html_arn = format!(
            "{}#GetHtmlError",
            create_local_state_machine_response
                .state_machine_arn()
                .unwrap()
        );
        let is_error_html_response = local_sfn_client
            .start_execution()
            .state_machine_arn(is_error_html_arn)
            .input(r#"{"url": "https://www.rust-lang.org/"}"#)
            .send()
            .await
            .unwrap();

        let is_error_html_result =
            Retry::spawn(strategy::FixedInterval::from_millis(500).take(4), || {
                assert_machine_status(
                    &local_sfn_client,
                    is_error_html_response.execution_arn().unwrap(),
                    aws_sdk_sfn::model::ExecutionStatus::Failed,
                )
            })
            .await;

        assert_eq!(is_error_html_result.is_err(), false);
        assert_eq!(is_error_html_result.unwrap(), "".to_string())
    }
}

async fn assert_machine_status(
    client: &aws_sdk_sfn::Client,
    arn: &str,
    status: aws_sdk_sfn::model::ExecutionStatus,
) -> Result<String, ()> {
    let execution = client
        .describe_execution()
        .execution_arn(arn)
        .send()
        .await
        .unwrap();

    if execution.status.clone().unwrap() == status {
        let execution_output = execution.output().unwrap_or("").to_string();
        return Ok(execution_output);
    }

    return Err(());
}
