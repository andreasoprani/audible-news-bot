# Audible News Bot (Rust version)

This version of the bot has been adapted to be deployed on AWS Lambda using S3 as a storage for settings, books and logs.

## Deployment from GitHub Actions

The repository includes `.github/workflows/deploy.yml`, a manually-triggered workflow that builds the Rust Lambda package and updates the existing AWS Lambda function code.

Required GitHub configuration:

- Repository variable `AWS_REGION`: AWS region of the Lambda, for example `eu-central-1`.
- Repository variable `LAMBDA_FUNCTION_NAME`: existing Lambda function name.
- Repository secret `AWS_GITHUB_ACTIONS_ROLE_ARN`: IAM role ARN that GitHub Actions can assume via OIDC.

The IAM role only needs permission to deploy code to this Lambda, for example `lambda:UpdateFunctionCode`, `lambda:GetFunction`, and `lambda:GetFunctionConfiguration` on the target function.

To deploy: GitHub → Actions → Deploy Lambda → Run workflow. Pick the same architecture as the existing Lambda (`x86_64` or `arm64`).

Future improvements:

- batch log writing at the end of the execution flow.
- batch book sending (leveraging async), timeouts from the telegram's API might be a problem.
- switch to something more sane for storing books (e.g. DynamoDB).
