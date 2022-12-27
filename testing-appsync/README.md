# Chapter 3 – Testing AppSync APIs

- As opposed to the APIGW VTL templates, **there is an API to test the AppSync VTL templates**.

  - This feature was released relatively recently. No need to string multiple internal VTL libraries together anymore!

  - Keep in mind that **the `evaluateMappingTemplate` SDK call has its limits**.

    - The most notable one is **the inability to parse the `context.prev` object in the context of pipeline resolvers**. They did not implement that one yet.

      - In such situations, **you have to fall back to using the `amplify` libraries from the _amplify simulator_**.

        - Keep in mind that these libraries are only available in JavaScript. Such a pity.

- **Another issue with the SDK calls is that they can take a bit longer to finish from time to time**.

  - Yan suspects that the AppSync team uses an AWS Lambda to evaluate the templates. The AWS Lambda might cold start, which could fail your test run.

- Integration and E2E tests follow a very similar structure to the ones written for APIGW.
