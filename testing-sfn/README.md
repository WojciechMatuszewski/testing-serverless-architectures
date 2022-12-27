# Chapter 4 – Testing Step Functions

- SFN are **notoriously hard to test due to the inability to start at a given state**.

  - The further away from the start, the state you want to test is, the harder it is to test it, as there are many pre-conditions to arrive at that state.

    - For such cases, the **step-functions-local Docker image is convenient** as it allows you to force a given step to fail.

- The **most problematic is the `Wait` state**. You could change the definition of the `Wait` state in step-functions-local, but that is changing the SFN definition for the purpose of testing. Not an ideal scenario.

  - It would be awesome if we could "fast forward" the `Wait` state somehow. As it stands, that is currently not possible.
