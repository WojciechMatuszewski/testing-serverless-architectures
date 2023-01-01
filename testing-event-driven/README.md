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
