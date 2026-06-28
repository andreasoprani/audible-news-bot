# Audible News Bot

Telegram bot that checks the Audible Italy catalogue for new audiobooks and posts updates to a Telegram channel.

The project now contains only the Rust implementation. It runs on AWS Lambda, is triggered by EventBridge, and uses S3 for persistent settings, stored books, and logs.

## Configuration

Runtime environment variables are listed in `bot.env`:

- `TELOXIDE_TOKEN`: Telegram bot token.
- `CHANNEL_ID`: Telegram channel/chat ID for public updates.
- `ADMIN_CHAT_ID`: Telegram chat ID that receives execution logs.
- `STORAGE_TYPE`: currently `S3`.
- `S3_BUCKET`: S3 bucket containing settings/books/log files.
- `S3_REGION`: S3 bucket region.
- `SETTINGS_JSON_FILE`: settings object key, usually `bot_settings.json`.
- `BOOKS_JSON_FILE`: stored books object key, usually `books.json`.
- `LOG_TXT_FILE`: log object key, usually `log.txt`.

`data/bot_settings.json` is a local/template copy of the settings file.

## Deployment from GitHub Actions

The repository includes `.github/workflows/deploy.yml`, a manually-triggered workflow that builds the Rust Lambda package and updates the existing AWS Lambda function code.

Required GitHub configuration:

- Repository variable `AWS_REGION`: AWS region of the Lambda, for example `eu-south-1`.
- Repository variable `LAMBDA_FUNCTION_NAME`: existing Lambda function name.
- Repository secret `AWS_GITHUB_ACTIONS_ROLE_ARN`: IAM role ARN that GitHub Actions can assume via OIDC.

The IAM role only needs permission to deploy code to this Lambda, for example `lambda:UpdateFunctionCode`, `lambda:GetFunction`, and `lambda:GetFunctionConfiguration` on the target function.

To deploy: GitHub → Actions → Deploy Lambda → Run workflow. The workflow currently builds for the default `x86_64` Lambda architecture.

## Data files

### `bot_settings.json`

- `url`: Audible advanced-search URL used to retrieve the latest catalogue additions.
- `url_header`: base URL used to complete book links.
- `max_books_kept`: number of books kept in `books.json`; should be greater than the page size.
- `max_message_length`: maximum Telegram message length.
- `default_log_length`: default number of log lines for log-related commands.
- `allowed_commands`: command names reserved by the bot.
- `attribute_names`: localized names used when formatting book messages.
- `redirect_message`: message for users who contact the bot directly.
- `book_url_message`: link text for Audible book details.

### `books.json`

Stores books already processed by the bot. Initialize as an empty list:

```json
[]
```

### `log.txt`

Execution log file.

## Future improvements

- Batch log writing at the end of the execution flow.
- Batch book sending; Telegram API timeouts might be a problem.
- Switch to a more suitable storage backend, e.g. DynamoDB.
