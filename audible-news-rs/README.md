# Audible News Bot (Rust version)

This version of the bot has been adapted to be deployed on AWS Lambda using S3 as a storage for settings, books and logs.

Future improvements:

- batch book sending (leveraging async), timeouts from the telegram's API might be a problem.
- switch to something more sane for storing books (e.g. DynamoDB).
