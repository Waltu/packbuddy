#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { PackingListStack } from "../lib/packing-list-stack";
import { DynamodbStack } from "../lib/dynamodb-stack";

const app = new cdk.App();
const dynamodbStack = new DynamodbStack(app, "DynamodbStack", {});
new PackingListStack(app, "PackingListStack", {
  dynamodbTable: dynamodbStack.dynamodbTable,
});
cdk.Tags.of(app).add("Project", "PackBuddy");
cdk.Tags.of(app).add("Environment", "Development");
