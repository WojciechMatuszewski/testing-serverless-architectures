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
