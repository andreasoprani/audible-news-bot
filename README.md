# Audible News Bot

This is a simple Telegram bot that searches for new additions to the Audible catalog and sends notifications to the users.

It currently supports only the italian version of Audible but it can be easily (I think) modified to support other Audible sites. Feel free to contact me if you wish to port this bot to your country's version of Audible.

## Libraries required

* beautifulsoup4
* lxml
* requests
* telepot

## Settings

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

## Token

The bot token given by Telegram should be stored in a file called 'token.txt', I obviously added it to .gitignore to prevent it from being uploaded to GitHub.

## Chats

The 'chats.json' file stores all the chats where the bot interacted, they are stored with their chat_id and a boolean called "active" that says if the bot should send the updates to the chat.
