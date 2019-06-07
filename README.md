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
* "url_header": header used for book links completion.
* "seconds_between_updates": number of seconds between each call of the update() function.
* "max_message_length": maximum length of a telegram message (4096 now).
* "allowed_image_formats": a list of allowed image formats (e.g. ".jpg").
* "allowed_commands": list of commands allowed in the chat with the bot, for each command these features are specified:
    * "command": correct syntax of the command ("/command").
    * "function": name of the function called by the message handler.
    * "description": brief description of the command, displayed to the player when the command /help is used.
    * "message": message sent to the user when the command is used.
* "attribute_names": attributes of the books, used to display them correctly in each language.
<<<<<<< HEAD
* "redirect_message": message sent when a not registered user sends a message to the bot.
* "no_image_text": text used when an image doesn't comply with the allowed formats.
* "book_url_message": message that will appear as a link to the book on the audible site.

### chats.json

* "admin_chat": id of the chat with the admin.
* "enabled_chats": list of the ids of all the chats that will receive the updates.

### token.txt

Insert your bot token in the "token.txt" file.