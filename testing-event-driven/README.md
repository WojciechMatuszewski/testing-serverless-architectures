# Chapter 5 – Testing Event-Driven Architectures

- There are many event-driven services in AWS. This makes writing tests challenging as you must consider each service's tradeoffs.

- Just like in the chapter related to SFN we have a problem of capturing the event data (in SFN case that was the `taskToken`).

  - Here, instead of an SQS queue, Yan **recommends using AppSync Subscriptions**. Quite an interesting choice.

    - Keep in mind that **EB does not integrate with AppSync directly. You will need to have an APIGW in front of the AppSync API**.

    - TODO: Would not invoking an HTTP Destination work on the same basis?

- Testing the DQL integration is another ball game.

  - You **could** remove permissions from the EventBridge, which is dangerous. If someone exists the tests before a cleanup (restore) action runs, your stack might be broken.

  - Instead, Yan suggests testing in production and relying on alerts.

- As I expected, using GraphQL with Rust is not as seamless as in JavaScript.

  - There does not seem to be a "de-facto" GraphQL client library.

- I **had to change the name of the zip archives to deploy multiple functions**.

  - [This comment helped a lot](https://github.com/serverless/serverless/issues/3696#issuecomment-559310048).

  - It baffles me that this is a problem. How is this not fixed yet?

- Unit testing the Rust AWS SDK client requires some boilerplate, but it mimics how one would test it using Go.

  - I'm aware of three possible ways to unit test the SDK.

    1. Use the [_trait objects_ approach](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/testing.html#testing-1).
    2. Use the [_enums_ approach](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/testing.html#testing-2).
    3. Use the [`TestConnection` approach](https://github.com/awslabs/aws-sdk-rust/issues/199#issuecomment-904558631).

    I'm leaning towards approach number 1 since it is the same as in Go. Approach number 2 is interesting (it is fascinating how feature-rich Rust enums are). I'm not so sure about the approach 3. It does seem like we are using implementation details.

  - The **worst part about unit testing** is that **you lose the ability** to use the `builder` pattern on the SDK calls**.
    One could probably implement it back, but it appears to be a lot of work.
