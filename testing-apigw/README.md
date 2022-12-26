# Chapter 2 – Strategy for testing API Gateway APIs

- There are A LOT of things to test in this seemingly simple setup.

  - Keep in mind that, apart from AWS Lambda, the APIGW can directly target services like DynamoDB or SNS.

  - There is also the authorization/authentication part, most likely implemented with Cognito.

- As for the direct integration – **at the time of writing this, there is no good way to test the APIGW VTL mapping templates**.

  - This saddens me as the support for testing AppSync VTL templates is excellent, especially after they added the SDK call.

- **Some features of APIGW are better tested on production**. These include _usage plans_, _throttling_, and so on.

- I kind of like the idea of `given`, `when`, and `then` modules for testing.

  - It makes it easy to add more and more test cases. I think I will adopt it in my projects.

- There **might be a significant overlap between the integration and e2e tests**, and that is **totally fine**.

  - These tests usually do not take that long to run. I wager it is better to wait a bit more and have some duplication than not having the tests at all!

- I'm not sure I'm a fan of using the [jest-runner-groups](https://www.npmjs.com/package/jest-runner-groups) package.

  - I understand the intent – getting rid of duplication, but we are doing so in the name of additional complexity.

## Overall thoughts

- Not much has changed in this landscape. The techniques Yan presents are the ones I'm using at work.

- I like the `given`, `when` and `then` module structure. It might be worth incorporating into my projects and then maybe showing that to my team at work.
