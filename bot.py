import time
import json
import telepot
from telepot.loop import MessageLoop
import utility
from datetime import date
from collections import OrderedDict

def start(chat_id, command):
    chats.append(chat_id)
    print(chats)
    print("\n")
    bot.sendMessage(chat_id, command["message"])

def stop(chat_id, command):
    chats.remove(chat_id)
    print(chats)
    print("\n")
    bot.sendMessage(chat_id, command["message"])
    
def help(chat_id, command):
    help_message = command["message"]
    
    for item in settings["allowed_commands"]:
        help_message += item["command"] + ": " + item["description"] + "\n"
        
    bot.sendMessage(chat_id, help_message)

def new(chat_id, command):
    new_message = command["message"]
    
    global updated
    global todayBooks
    
    if updated != date.today():
        todayBooks = utility.getNewBooks(settings["url"])
        updated = date.today()
    
    new_message += utility.messageBuilder(todayBooks, settings["attribute_names"])
    
    bot.sendMessage(chat_id, new_message)

def handle(msg):
    content_type, chat_type, chat_id, date, message_id = telepot.glance(msg, long=True)
    text = msg["text"]
    
    print("New message", 
          "Chat ID: " + str(chat_id), 
          "Chat type: " + chat_type, 
          "Content type: " + content_type, 
          "Text: " + text, 
          "Date: " + str(date), 
          "Message ID: " + str(message_id), 
          "\n")
   
   # if not text then dump
    if content_type != "text": return

    # Call correct function
    for item in settings["allowed_commands"]:
        if item["command"] == text:
            func = item["function"]
            globals()[func](chat_id, item)


def main():
    
    global settings
    
    # Get token from token file.
    try:
        token_file = open('token.txt', 'r')
        TOKEN = token_file.read()
        token_file.close()
    except FileNotFoundError:
        print("Token file not found.")
        return

    # Get settings from settings file
    try:
        settings_file = open("bot_settings.json")
        settings = json.load(settings_file, object_pairs_hook=OrderedDict)
        settings_file.close()
    except FileNotFoundError:
        print("Settings file not found.")
        return
    
    # Create the bot.
    global bot
    bot = telepot.Bot(TOKEN)
    
    # Chats where the bot is activated.
    global chats
    chats = []
    
    # Books of the day   
    global todayBooks
    todayBooks = utility.getNewBooks(settings["url"])
    
    # Day of the last update of todayBooks
    global updated
    updated = date.today()
    
    MessageLoop(bot, handle).run_as_thread()
    print("Listening...")
    
    while(1):
        time.sleep(10)

if __name__ == "__main__":
    main()