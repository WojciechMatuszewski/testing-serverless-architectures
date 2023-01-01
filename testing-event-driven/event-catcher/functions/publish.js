require("isomorphic-fetch");
const gql = require("graphql-tag");
const { AWSAppSyncClient, AUTH_TYPE } = require("aws-appsync");
const ulid = require("ulid");
const AWS = require("aws-sdk");
const axios = require("axios");

const client = new AWSAppSyncClient({
  url: process.env.API_URL,
  region: process.env.AWS_REGION,
  auth: {
    type: AUTH_TYPE.API_KEY,
    apiKey: () => "xxx"
  },
  disableOffline: true
});

/**
 *
 * @param {import('aws-lambda').APIGatewayEvent} event
 * @returns {Promise<import('aws-lambda').APIGatewayProxyResult>}
 */
module.exports.handler = async event => {
  const { accountId, target } = event.pathParameters;

  // handle SNS subscription confirmations
  if (event.headers["x-amz-sns-message-type"] === "SubscriptionConfirmation") {
    console.log("confirming SNS subscription");

    const snsReq = JSON.parse(event.body);
    await axios(snsReq.SubscribeURL);

    return {
      statusCode: 200
    };
  }

  const resp = await client.mutate({
    mutation: gql`
      mutation publish($event: EventInput!) {
        publish(event: $event) {
          accountId
          eventId
          target
          payload
        }
      }
    `,
    variables: {
      event: {
        accountId,
        eventId: ulid.ulid(),
        target,
        payload: event.body
      }
    }
  });

  const newEvent = resp.data.publish;

  return {
    statusCode: 200,
    body: JSON.stringify(newEvent)
  };
};
