# Audible News Bot

This is a simple Telegram bot that searches for new additions to the Audible catalog and sends notifications to the users.

It currently supports only the italian version of Audible but it can be easily (I think) modified to support other Audible sites. Feel free to contact me if you wish to port this bot to your country's version of Audible.

## Libraries required

* beautifulsoup4
* lxml
* requests
* telepot

## Settings

### bot_settings.json

* "url": advanced search url used to retrieve the latest additions to the Audible catalogues, the settings used were:
    * Only Audiobooks.
    * Sort by newest arrivals (important).
    * 50 titles per page (important on frequently updated catalogue, this creates a problem if the more than 50 titles are added to the catalogue at once).
* "seconds_between_updates": number of seconds between each call of the update() function.
* "allowed_commands": list of commands allowed in the chat with the bot, for each command these features are specified:
    * "command": correct syntax of the command ("/command").
    * "function": name of the function called by the message handler.
    * "description": brief description of the command, displayed to the player when the command /help is used.
    * "message": message sent to the user when the command is used.
* "attribute_names": attributes of the books, used to display them correctly in each language.
* "redirect_message": message sent when a not registered user sends a message to the bot.

### chats.json

Insert here all the registered chats that the bot can interact with. Each element must have two attributes:

* "chat_id": id of the chat.
* "active": whether the chat receives updates. This attribute can be modified with the commands /start and /stop and checked with /status.

### token.txt

Insert your bot token in the "token.txt" file.