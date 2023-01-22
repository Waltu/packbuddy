import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as path from "path";
import {
  Code,
  Function,
  Runtime,
  FunctionUrlAuthType,
} from "aws-cdk-lib/aws-lambda";
import { CfnOutput } from "aws-cdk-lib";
import { RustFunction } from "rust.aws-cdk-lambda";

interface StackProps extends cdk.StackProps {
  dynamodbTable: cdk.aws_dynamodb.ITable;
}

export class PackingListStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: StackProps) {
    super(scope, id, props);

    const dynamodbTable = props.dynamodbTable;

    const PACKING_LIST_BY_ID_NAME = "get-packing-list-by-id";
    const handler = new RustFunction(this, "GetPackingListById", {
      directory: path.join(
        __dirname,
        "..",
        "..",
        `services/${PACKING_LIST_BY_ID_NAME}`
      ),
      architecture: cdk.aws_lambda.Architecture.ARM_64,
      tracing: cdk.aws_lambda.Tracing.ACTIVE,
      description: "Get a packing list by ID",
      logRetention: cdk.aws_logs.RetentionDays.ONE_WEEK,
      insightsVersion: cdk.aws_lambda.LambdaInsightsVersion.VERSION_1_0_135_0,
      environment: {
        DYNAMODB_TABLE_NAME: dynamodbTable.tableName,
      },
    });

    dynamodbTable.grantReadData(handler);

    const fnUrl = handler.addFunctionUrl({
      authType: FunctionUrlAuthType.NONE,
    });

    new CfnOutput(this, `${PACKING_LIST_BY_ID_NAME}-url`, {
      value: fnUrl.url,
    });
  }
}
