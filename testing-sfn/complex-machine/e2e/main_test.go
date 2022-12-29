package main_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/avast/retry-go"
	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/sfn"
	sfntypes "github.com/aws/aws-sdk-go-v2/service/sfn/types"
	"github.com/aws/aws-sdk-go-v2/service/sqs"
	"github.com/google/uuid"
	"github.com/joho/godotenv"
)

func TestMain(m *testing.M) {
	err := godotenv.Load("../.outputs.env")
	if err != nil {
		panic(fmt.Errorf("Could not load .outputs.env: %w", err))
	}

	os.Exit(m.Run())
}

func TestWorkflow(t *testing.T) {
	stateMachineArn := os.Getenv("OrdersMachineArn")
	ctx := context.Background()

	// TODO: check in DDB
	t.Run("Order accepted flow", func(t *testing.T) {
		stopPollingForOrderedOrders, orderedOrders := pollForOrderMessages(ctx, os.Getenv("OrderedOrdersQueueUrl"))
		stopPollingForAcceptedOrders, acceptedOrders := pollForOrderMessages(ctx, os.Getenv("AcceptedOrdersQueueUrl"))

		config, err := config.LoadDefaultConfig(ctx)
		if err != nil {
			panic(fmt.Errorf("Could not load config: %w", err))
		}

		orderId := uuid.New().String()
		sfnClient := sfn.NewFromConfig(config)
		out, err := sfnClient.StartExecution(ctx, &sfn.StartExecutionInput{
			StateMachineArn: aws.String(stateMachineArn),
			Input:           aws.String(fmt.Sprintf("{\"orderId\": \"%s\"}", orderId)),
		})
		if err != nil {
			panic(fmt.Errorf("Could not start execution: %w", err))
		}
		executionArn := out.ExecutionArn

		for orderedOrders := range orderedOrders {
			for _, orderedOrder := range orderedOrders {
				if orderedOrder.OrderId != orderId {
					continue
				}

				_, err = sfnClient.SendTaskSuccess(ctx, &sfn.SendTaskSuccessInput{
					Output: aws.String(
						fmt.Sprintf("{\"orderId\": \"%s\", \"accepted\": true}", orderedOrder.OrderId),
					),
					TaskToken: aws.String(orderedOrder.TaskToken),
				})
				if err != nil {
					panic(fmt.Errorf("Could not send task success: %w", err))
				}

				stopPollingForOrderedOrders()
			}
		}

		fmt.Println("Order accepted! Now waiting for the fulfillment")

		for acceptedOrders := range acceptedOrders {
			for _, acceptedOrder := range acceptedOrders {
				if acceptedOrder.OrderId != orderId {
					continue
				}
				_, err = sfnClient.SendTaskSuccess(ctx, &sfn.SendTaskSuccessInput{
					Output: aws.String(
						fmt.Sprintf("{\"orderId\": \"%s\"}", acceptedOrder.OrderId),
					),
					TaskToken: aws.String(acceptedOrder.TaskToken),
				})
				if err != nil {
					panic(fmt.Errorf("Could not send task success: %w", err))
				}

				stopPollingForAcceptedOrders()
			}
		}

		fmt.Println("Order fulfilled! Checking the execution status")

		err = retry.Do(func() error {
			out, err := sfnClient.DescribeExecution(ctx, &sfn.DescribeExecutionInput{
				ExecutionArn: aws.String(*executionArn),
			})
			if err != nil {
				return fmt.Errorf("Could not describe execution: %w", err)
			}

			if out.Status != sfntypes.ExecutionStatusSucceeded {
				return fmt.Errorf("Execution is not succeeded: %s", out.Status)
			}

			return nil
		}, retry.Attempts(5), retry.Delay(1*time.Second))
		if err != nil {
			panic(fmt.Errorf("Could not describe execution: %w", err))
		}

		fmt.Println("All done!")

	})
}

type OrderedOrderMessage struct {
	Message string `json:"Message"`
}

type OrderedOrderMessagePayload struct {
	TaskToken string `json:"taskToken"`
	OrderId   string `json:"orderId"`
}

func pollForOrderMessages(ctx context.Context, queueUrl string) (func(), chan []OrderedOrderMessagePayload) {
	config, err := config.LoadDefaultConfig(ctx)
	if err != nil {
		panic(fmt.Errorf("Could not load config: %w", err))
	}
	sqsClient := sqs.NewFromConfig(config)

	receivedMessages := make(map[string]OrderedOrderMessagePayload)
	getMessages := func() {
		out, err := sqsClient.ReceiveMessage(ctx, &sqs.ReceiveMessageInput{
			QueueUrl: aws.String(queueUrl),
		})
		if err != nil {
			panic(fmt.Errorf("Could not receive message: %w", err))
		}

		for _, message := range out.Messages {
			var orderedOrderMessage OrderedOrderMessage
			err = json.Unmarshal([]byte(*message.Body), &orderedOrderMessage)
			if err != nil {
				panic(fmt.Errorf("Could not unmarshal message: %w", err))
			}

			var orderedOrderMessagePayload OrderedOrderMessagePayload
			err = json.Unmarshal([]byte(orderedOrderMessage.Message), &orderedOrderMessagePayload)
			if err != nil {
				panic(fmt.Errorf("Could not unmarshal message payload: %w", err))
			}

			receivedMessages[*message.MessageId] = orderedOrderMessagePayload
		}
	}

	ticker := time.NewTicker(2 * time.Second)
	quit := make(chan struct{}) // The `ticket` does not close the channel when the `.close` is called.
	results := make(chan []OrderedOrderMessagePayload)

	go func() {
		for {
			select {
			case <-ticker.C:
				getMessages()
				var orderedOrderMessagePayloads []OrderedOrderMessagePayload
				for _, v := range receivedMessages {
					orderedOrderMessagePayloads = append(orderedOrderMessagePayloads, v)
				}
				results <- orderedOrderMessagePayloads
			case <-quit:
				return
			}
		}
	}()

	return func() {
		ticker.Stop()
		close(quit)
		close(results)
	}, results

}
