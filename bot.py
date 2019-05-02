import time
import json
import telepot
from telepot.loop import MessageLoop
from pprint import pprint

def handle(msg):
    content_type, chat_type, chat_id = telepot.glance(msg)
    
    print("content_type: " + content_type)
    print("chat_type: " + chat_type)
    print("chat_id: " + str(chat_id))

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
        settings = json.load(settings_file)
        settings_file.close()
    except FileNotFoundError:
        print("Settings file not found.")
        return
    
    # Create the bot.
    bot = telepot.Bot(TOKEN)
    
    # Chats where the bot is activated.
    global chats = []
    
    MessageLoop(bot, handle).run_as_thread()
    print("Listening...")
    
    while(1):
        time.sleep(10)

if __name__ == "__main__":
    main()