# Testing Serverless Architectures course

Going through the course and writing what I've learned along the way.

## Chapter 1 – How to test Serverless architectures

- Unit tests for the domain logic, integration tests for the _adapters_, and e2e tests for the whole system.

  - Fully agree.

- You probably do not need to use the hexagonal architecture in its "true" form. Keeping the "core domain" as a layer in most projects will do.

- The integration tests and unit tests are run locally. You should favor the actual services whenever possible (such tests give you much more confidence).

  - Keep in mind that, as far as I know, _localstack_ and similar services are re-implementation of the original AWS services. How much confidence do you have that there are no bugs there?

- I do not have experience with _localstack_, but Yan says it can be pretty brittle. I can imagine this being the case.

- While running locally against the actual AWS services, you can still use the debugger if you wish.

  - **Running against the actual AWS services makes it very hard to test rare edge cases**** like throttling and other network-related issues.

    - There is the AWS FIS, but that works only in the context of the VPC.

    - Here, **consider using mocks**.

- Yan correctly states that if you have trouble debugging your e2e/integration tests, you will have a much bigger problem when the production system is down.

  - Keep your logs clean. Understand where to look for them. Know your tools.

- Yan presents the _testing honeycomb_ – a prevalent way of testing applications in the serverless land.

  - That is because the **unit tests do not give you a lot of return on investment in serverless architectures**.

    - Of course, having the unit tests for the service edge cases and domain logic is a must, but other than that, it should be integration tests all the way down (with a few e2e tests here and there).

- **For authentication, use the _admin_ version of the Cognito SDK**.

  - It allows you to create and confirm users. Very handy when writing tests.
