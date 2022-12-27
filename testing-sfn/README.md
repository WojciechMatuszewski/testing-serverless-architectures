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
