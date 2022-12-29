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
	dynamodbattributevalue "github.com/aws/aws-sdk-go-v2/feature/dynamodb/attributevalue"
	"github.com/aws/aws-sdk-go-v2/service/dynamodb"
	dynamodbtypes "github.com/aws/aws-sdk-go-v2/service/dynamodb/types"
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

	t.Run("Order accepted flow", func(t *testing.T) {
		stopPollingForOrderedOrders, orderedOrders := pollForOrderMessages(ctx, os.Getenv("OrderedOrdersQueueUrl"))
		stopPollingForAcceptedOrders, acceptedOrders := pollForOrderMessages(ctx, os.Getenv("AcceptedOrdersQueueUrl"))

		config, err := config.LoadDefaultConfig(ctx)
		if err != nil {
			panic(fmt.Errorf("Could not load config: %w", err))
		}

		dynamoDBClient := dynamodb.NewFromConfig(config)

		orderId := uuid.New().String()
		sfnClient := sfn.NewFromConfig(config)
		out, err := sfnClient.StartExecution(ctx, &sfn.StartExecutionInput{
			StateMachineArn: aws.String(stateMachineArn),
			Input:           aws.String(fmt.Sprintf("{\"orderId\": \"%s\"}", orderId)),
		})
		if err != nil {
			t.Fatal(fmt.Errorf("Could not start execution: %w", err))
		}
		executionArn := out.ExecutionArn

		fmt.Println("Order ordered! checking in the database")

		err = retry.Do(func() error {
			order, err := readOrder(ctx, dynamoDBClient, orderId)
			if err != nil {
				return err
			}
			if order.Status != "ORDERED" {
				return fmt.Errorf("Order is not ordered: %s", order.Status)
			}
			return nil
		}, retry.Attempts(5), retry.Delay(500*time.Millisecond))
		if err != nil {
			t.Fatal(err)
		}

		fmt.Println("Order ordered in the database! Checking the messages")

		for currentOrderedOrders := range orderedOrders {
			for _, orderedOrder := range currentOrderedOrders {
				fmt.Println("Current ordered orders", currentOrderedOrders)

				if orderedOrder.OrderId != orderId {
					continue
				}

				fmt.Println("Accepting the order", orderId)
				_, err = sfnClient.SendTaskSuccess(ctx, &sfn.SendTaskSuccessInput{
					Output: aws.String(
						fmt.Sprintf("{\"orderId\": \"%s\", \"accepted\": true}", orderedOrder.OrderId),
					),
					TaskToken: aws.String(orderedOrder.TaskToken),
				})
				if err != nil {
					t.Fatal(fmt.Errorf("Could not send task success: %w", err))
				}

				stopPollingForOrderedOrders()
			}
		}

		fmt.Println("Order ordered in the database! Waiting for acceptation")

		for currentAcceptedOrders := range acceptedOrders {
			for _, acceptedOrder := range currentAcceptedOrders {
				if acceptedOrder.OrderId != orderId {
					continue
				}

				fmt.Println("Confirming order acceptation", orderId)
				_, err = sfnClient.SendTaskSuccess(ctx, &sfn.SendTaskSuccessInput{
					Output: aws.String(
						fmt.Sprintf("{\"orderId\": \"%s\"}", acceptedOrder.OrderId),
					),
					TaskToken: aws.String(acceptedOrder.TaskToken),
				})
				if err != nil {
					t.Fatal(fmt.Errorf("Could not send task success: %w", err))
				}

				stopPollingForAcceptedOrders()
			}
		}

		fmt.Println("Order accepted! checking in the database")

		err = retry.Do(func() error {
			order, err := readOrder(ctx, dynamoDBClient, orderId)
			if err != nil {
				return err
			}
			if order.Status != "ACCEPTED" {
				return fmt.Errorf("Order is not accepted: %s", order.Status)
			}
			return nil
		}, retry.Attempts(5), retry.Delay(500*time.Millisecond))
		if err != nil {
			t.Fatal(err)
		}

		fmt.Println("Order accepted in the database! Checking execution status")

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
			t.Fatal(fmt.Errorf("Could not describe execution: %w", err))
		}

	})

	t.Run("Order rejected flow", func(t *testing.T) {
		stopPollingForOrderedOrders, orderedOrders := pollForOrderMessages(ctx, os.Getenv("OrderedOrdersQueueUrl"))
		stopPollingForRejectedOrders, rejectedOrders := pollForOrderMessages(ctx, os.Getenv("RejectedOrdersQueueUrl"))

		config, err := config.LoadDefaultConfig(ctx)
		if err != nil {
			panic(fmt.Errorf("Could not load config: %w", err))
		}

		dynamoDBClient := dynamodb.NewFromConfig(config)

		orderId := uuid.New().String()
		fmt.Println("OrderId", orderId)

		sfnClient := sfn.NewFromConfig(config)
		out, err := sfnClient.StartExecution(ctx, &sfn.StartExecutionInput{
			StateMachineArn: aws.String(stateMachineArn),
			Input:           aws.String(fmt.Sprintf("{\"orderId\": \"%s\"}", orderId)),
		})
		if err != nil {
			t.Fatal(fmt.Errorf("Could not start execution: %w", err))
		}
		executionArn := out.ExecutionArn

		fmt.Println("Order ordered! checking in the database")

		err = retry.Do(func() error {
			order, err := readOrder(ctx, dynamoDBClient, orderId)
			fmt.Println("Checking for ordered order", order, err)

			if err != nil {
				return err
			}
			if order.Status != "ORDERED" {
				return fmt.Errorf("Order is not ordered: %s", order.Status)
			}
			return nil
		}, retry.Attempts(5), retry.Delay(500*time.Millisecond))
		if err != nil {
			t.Fatal(err)
		}

		for currentOrderedOrders := range orderedOrders {
			for _, orderedOrder := range currentOrderedOrders {
				if orderedOrder.OrderId != orderId {
					continue
				}

				_, err = sfnClient.SendTaskSuccess(ctx, &sfn.SendTaskSuccessInput{
					Output: aws.String(
						fmt.Sprintf("{\"orderId\": \"%s\", \"accepted\": false}", orderedOrder.OrderId),
					),
					TaskToken: aws.String(orderedOrder.TaskToken),
				})
				if err != nil {
					t.Fatal(fmt.Errorf("Could not send task success: %w", err))
				}

				stopPollingForOrderedOrders()
			}
		}

		fmt.Println("Order rejected! checking in the database")

		err = retry.Do(func() error {
			order, err := readOrder(ctx, dynamoDBClient, orderId)
			fmt.Println("Checking for rejected orders", order, err)
			if err != nil {
				return err
			}
			if order.Status != "REJECTED" {
				return fmt.Errorf("Order is not rejected: %s", order.Status)
			}

			return nil
		}, retry.Attempts(5), retry.Delay(500*time.Millisecond))
		if err != nil {
			t.Fatal(err)
		}

		fmt.Println("Order rejected in the database! Checking notifications")

		for currentRejectedOrders := range rejectedOrders {
			fmt.Println("Current rejected orders", currentRejectedOrders)

			for _, rejectedOrder := range currentRejectedOrders {
				if rejectedOrder.OrderId != orderId {
					continue
				}

				stopPollingForRejectedOrders()
			}
		}

		fmt.Println("User notified about the rejection. Checking execution status")

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
			t.Fatal(fmt.Errorf("Could not describe execution: %w", err))
		}

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

type Order struct {
	OrderId string `json:"orderId" dynamodbav:"orderId"`
	Status  string `json:"status" dynamodbav:"status"`
}

func readOrder(ctx context.Context, client *dynamodb.Client, orderId string) (Order, error) {
	var order Order
	out, err := client.GetItem(ctx, &dynamodb.GetItemInput{
		TableName: aws.String(os.Getenv("OrdersTableName")),
		Key: map[string]dynamodbtypes.AttributeValue{
			"orderId": &dynamodbtypes.AttributeValueMemberS{
				Value: orderId,
			},
		},
	})
	if err != nil {
		return order, err
	}

	err = dynamodbattributevalue.UnmarshalMap(out.Item, &order)
	if err != nil {
		return order, err
	}

	return order, nil
}
