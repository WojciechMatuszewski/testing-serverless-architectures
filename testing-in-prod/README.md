# Chapter 6 – Testing in Production

- It is not about running e2e/integration tests on the production environment. **It is about running our changes against a small portion of the live traffic**.

  - One could use **canary deployments** to achieve this.

    - Think using the rolling AWS Lambda deployments with CodeDeploy service.

      - This strategy is **not without drawbacks**. Here are a few mentioned by Yan:

        1. Traffic is split by requests, not users (not users are equal). This has huge implications for downstream services that might call your API again. Imagine two calls to your API (in the same "pipeline") that invoke different API versions.

        2. Monitoring and alerting might be problematic, especially around latency. Keep in mind that the "new" version is invoked less. Thus the cold starts might occur more often.

    - There are a **few alternatives to consider when it comes to canary deployments without CodeDeploy/Lambda aliases**:

        1. Splitting the requests on the client level. This way, we always split the traffic based on the user.

        2. **Feature flags that are "user-context" aware**.

- Note that **canary deployments are not the same as blue/green deployments**.

  - In **blue/green** scenario, you **switch all the traffic to the new version but keep the old one around if rollback is necessary**.

  - In the **canary deployment** scenario, you **forward a small portion of your traffic to the new version, slowly increasing its amount**.

- Let us not forget about **the smoke tests**. These could act as a "sanity check" for the core features of your application.

  - They run after the deployment. **You can perform them manually if necessary** after you ship your feature.

    - I'm not sure I like the idea of manual tests, but they are better than nothing.

- As for the **drawbacks of the feature flags** – there are some. Here is a noncomprehensive list:

    1. Feature flags add additional complexity to your code. Look at all the `if` statements and different code branches.

    2. There is a **performance overhead to think about**. Every AWS Lambda function needs to ask the feature flag service about the application state and experiments. What if that service fails?

- **Running load tests could be expensive**, so it is vital to have an end goal in mind while conducting those.

  - First and foremost, ensure that the traffic pattern you replicate is a realistic one. There is no point in hammering the `login` endpoint if your users only need to log in once a couple of days/weeks. That is an excellent tip from Yan!

- **The ecosystem of chaos engineering in the context of serverless is not mature yet**.

  - There are NO "official" tools for injecting failures into the AWS Lambda. There is one package called ["failure-lambda"](https://github.com/gunnargrosch/failure-lambda).

  - AWS has the [AWS FIS service](https://aws.amazon.com/fis/), but it focuses on containers, EC2, or in-VPC networking like similar services.

  - **Yan recommends going multi-region and testing the failover**. Since there are no servers to kill (at least not the ones we would manage), it is hard to devise different testing strategies.

- In the chapter about observability, **Yan mentions that logs are overrated**.

  - In a large system, the number of logs will be huge, especially when there is no discipline in what should or should not be logged (as it usually is). It could feel like searching for a needle in a haystack.

    - It is **worth noting that you cannot log everything, especially PII data**. This makes it tough to devise a clear and concise logging strategy.

      - Ideally, we could log all the requests we make to the AWS services via the SDK. But if we were to do that, we would put PII data into CW, which is a no-no in most cases.

        - You **have to have a data-filtering mechanism for your logs in place!**

- AWS has an X-Ray service, but its use cases are limited.

  - My most significant issues with X-Ray are that it does not capture request/response bodies and that it's filtering capabilities are hard to use.

  - 3rd party applications are better in this regard.
