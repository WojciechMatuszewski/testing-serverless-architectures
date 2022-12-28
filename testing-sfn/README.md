# Chapter 4 – Testing Step Functions

- SFN are **notoriously hard to test due to the inability to start at a given state**.

  - The further away from the start, the state you want to test is, the harder it is to test it, as there are many pre-conditions to arrive at that state.

    - For such cases, the **step-functions-local Docker image is convenient** as it allows you to force a given step to fail.

- The **most problematic is the `Wait` state**. You could change the definition of the `Wait` state in step-functions-local, but that is changing the SFN definition for the purpose of testing. Not an ideal scenario.

  - It would be awesome if we could "fast forward" the `Wait` state somehow. As it stands, that is currently not possible.

- It seems like the `AWS_REGION` and `AWS_DEFAULT_REGION` environment variables are not supported in Serverless framework. I'm surprised that this is the case. [Link to the issue](https://github.com/serverless/serverless/issues/2151).

- The SFN local package is not so user-friendly. The format of the mocked responses is quite weird.

  - Keep in mind that, with the SFN local, all the negatives related to local testing still apply.

    - It does not simulate IAM.
    - Not all services can be mocked.
    - It is pretty brittle.

- For **more complex workflows, you will most likely need to deploy a "test-only" infrastructure, like SQS**.

  - How else could you control the _waitForCallback_ flow? You need to deploy some resources to read the token from.

    - Okay, you might get away with not deploying any resource if you save the tokens in DDB, but you will still need to read from it.

  - Implementing custom SQS pollers is a bit of a pain, but I do not see any other way to test the SNS -> _waitForCallback_ pattern than to use SQS. Yan is using the same technique.

- When **testing timeouts, Yan decided to re-write the SFN definition when providing it to the step functions local**.

  - Interesting technique. It smells, but I do not have a better solution either.

- For **testing 3rd party APIs**, instead of using the step functions local, **consider creating mock endpoints to force the response you want**.

  - In the lecture, Yan uses `ngrok` to create a local public endpoint. If you run the tests locally, you **also could get away with using `msw` in some cases**.

- I could not find a way to use the `waitForTaskToken` with a non-SDK call state transition.

  - IMO, it would be neat if one could use a `Pass` state to do so, but maybe it is not necessary?

    - Maybe trying to do so is a sign of a bad design?

- And, of course, I forgot about the `AWS::SQS::QueuePolicy` resource.

  - It took me longer than I would like to figure out the resource was missing.
