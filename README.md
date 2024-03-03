# Audible News Bot

This is a simple Telegram bot that searches for new additions to the Audible catalog and sends them to some chats.

It currently supports only the italian version of Audible but it can be easily modified to support other Audible sites. Feel free to contact me if you wish to port this bot to your country's version of Audible.

This project was initially written in Python, it has been rewritten in Rust (as a personal exercise). Both versions are available in this repo and they work pretty much in the same way (except for the fact that the Rust version doesn't accept commands yet and that it runs on AWS Lambda instead of Docker).

## Running

### Python Version

The bot is containerized and it can be run with:

```
docker-compose up --build -d
```

In order to run it, some environment variables have to be included in a .env file (`bot.env`), these variables are:

Necessary:

- `TOKEN`: the telegram api token.
- `ADMIN_CHAT`: the id of the chat between the bot and the admin.
- `CHANNEL_CHAT`: the id of the channel chat where the bot will send the updates.

Optional (in order of priority):

- `MINUTES_INTERVAL`: if this is set, the bot will be run every `MINUTES_INTERVAL` minutes.
- `HOURS_INTERVAL`: if this is set, the bot will be run every `HOURS_INTERVAL` hours.
- `DAYS_INTERVAL`: if this is set, the bot will be run every `DAYS_INTERVAL` days.

If none of these are set, the bot will be run every hour.

### Rust version

This is the latest version of the bot, it has been adapted to run on AWS Lambda (built and deployed through `cargo lambda`) using S3 for storing the persistent data.

All the environment variables needed to run it are specified in the `bot.env` file.

## Data

The `data` directory stores some information both relevant to the bot and produced by it during its execution.

### bot_settings.json

- `url`: advanced search url used to retrieve the latest additions to the Audible catalogues, the settings used were:
  - Only Audiobooks.
  - Sort by newest arrivals (important).
  - 50 titles per page (important on frequently updated catalogue, this creates a problem if the more than 50 titles are added to the catalogue at once).
- `url_header`: header used for book links completion.
- `max_books_kept`: number of books kept in the books.json file, for a correct execution of the bot it should be >50.
- `max_message_length`: maximum length of a telegram message (4096 now).
- `default_log_length`: default number of log lines sent when the admin sends the /log command without specifying the number of lines requested
- `allowed_image_formats`: a list of allowed image formats (e.g. `.jpg`).
- `allowed_commands`: list of commands allowed in the chat with the bot, for each command these features are specified:
  - `command`: correct syntax of the command (`/command`).
  - `function`: name of the function called by the message handler.
  - `description`: brief description of the command, displayed to the player when the command /help is used.
  - `message`: message sent to the user when the command is used.
- `attribute_names`: attributes of the books, used to display them correctly in each language.
- `redirect_message`: message sent when a not registered user sends a message to the bot.
- `no_image_text`: text used when an image doesn't comply with the allowed formats.
- `book_url_message`: message that will appear as a link to the book on the audible site.

### bot_state.json

This is relevant only for the python version for now.

- `last_udpate`: stores the last message id sent by the admin, set as 0 at the beginning.
- `active`: stores a boolean that tells if the bot should check for updates or not. It is changed with the /start and /stop commands.

### books.json

It stores all the books found, up to a number defined in the settings. To be initialized as an empty list, i.e. `[]`.

### log.txt

The complete log of the bot.
