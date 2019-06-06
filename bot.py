# Standard libraries
import time
import json
import datetime
from collections import OrderedDict
import re

# Bot related libraries
import telepot
from telepot.loop import MessageLoop

# HTML parsing related libraries
import requests
from bs4 import BeautifulSoup


def sendBook(chat_id, book):
    """Send a specific book to a specific chat."""
    
    # Build the message
    message = ""
    for key in settings["attribute_names"].keys():
        message += settings["attribute_names"][key] + ": " + str(book[key]) + "\n"
    
    url = settings["url_header"] + book["URL"]
    message += "[" + settings["book_url_message"] + "](" + url + ")"
    
    # Send the book
    if book["imageURL"][-4:].lower() not in settings["allowed_image_formats"]:
        message += "Immagine non disponibile."
        bot.sendMessage(chat_id, text = message)
    else:
        bot.sendPhoto(chat_id, book["imageURL"], caption=message,parse_mode="Markdown")

def sendBookToAll(book):
    """Send a specific book to all the active chats."""
    
    # Retrieve chats
    with open("chats.json") as f:
        chats = json.load(f)
    
    # Send book to every chat
    for chat in chats["enabled_chats"]:
        sendBook(chat, book)

def update():
    """
    This function is used to check for update in the Audible catalog.
    It checks if there are new books at the url given in the settings and it sends the latest books 
    to all the active chats.
    """
    
    # Load today books
    with open("today_books.json") as f:
        todayBooks = json.load(f)
    
    # Get today date and confront
    today = datetime.date.today()
    
    # If the last update was from yesterday, set todayBooks as blank and to today
    if todayBooks["date"] != str(today):
        todayBooks["date"] = str(today)
        todayBooks["books"] = []
        
        with open("today_books.json", "w") as f:
            json.dump(todayBooks, f, indent=4, separators=(',', ': '))
    
    # GET request
    content = requests.get(settings["url"]).content

    # BS data structure
    soup = BeautifulSoup(content, features="lxml")

    # Retrieve books
    productList = soup.findAll("li", "productListItem")

    # Check date and get info
    for product in productList:

        # Check date
        dateString = product.find("li", "releaseDateLabel").find("span").contents[0]
        book_date = datetime.date(*[int(s) for s in dateString.split() if s.isdigit()][::-1])
        if book_date != today:
            break
    
        # Get info
        book_title = product.get("aria-label") #.encode('latin1').decode('utf-8')
        book_author = product.find("li", "authorLabel").find("a").contents[0] #.encode('latin1').decode('utf-8')
        book_narrator = product.find("li", "narratorLabel").find("a").contents[0] #.encode('latin1').decode('utf-8')
        book_runtime = product.find("li", "runtimeLabel").find("span").contents[0].replace("Durata:  ","")
        book_imageURL = product.find("img", "bc-image-inset-border").get("src")
        book_URL = product.find("a", "bc-link").get("href")

        # Create book dict
        book = {
            "title": book_title,
            "author": book_author,
            "narrator": book_narrator,
            "runtime": book_runtime,
            "date": str(book_date),
            "imageURL": book_imageURL,
            "URL": book_URL
        }
        
        # Add new books to the books list and send it to everyone
        if book not in todayBooks["books"]:
            todayBooks["books"].append(book)
            
            with open("log.txt","a+") as log:
                log.write(str(datetime.datetime.now()) + " - " + str(book) + "\n")
                
            sendBookToAll(book)
    
    # Update today_books file
    with open('today_books.json', 'w') as f:
        json.dump(todayBooks, f, indent=4, separators=(',', ': '))

def start(chat_id, text, command):
    """
    This function is called when the admin uses the /start command.
    It starts updates.
    """
    
    # Activate updates
    global active
    active = True
    
    # Log
    with open("log.txt","a+") as log:
        log.write(str(datetime.datetime.now()) + " - Updates activated.\n")
    
    bot.sendMessage(chat_id, command["message"])

def stop(chat_id, text, command):
    """
    This function is called when the admin uses the /stop command.
    It stops updates.
    """
    
    # De-activate updates
    global active
    active = False
    
    # Log
    with open("log.txt","a+") as log:
        log.write(str(datetime.datetime.now()) + " - Updates de-activated.\n")
    
    bot.sendMessage(chat_id, command["message"])
    
def help(chat_id, text, command):
    """
    This function is called whenever a user uses the /help command.
    It sends a help message to the chat.
    """
    
    help_message = command["message"]
    
    for item in settings["allowed_commands"]:
        help_message += item["command"] + ": " + item["description"] + "\n"
        
    bot.sendMessage(chat_id, help_message)

def log(chat_id, text, command):
    """
    This function is called whenever the admin uses the /log command.
    It sends the log of the bot.
    """

    # Number of lines requested
    lines_number = int(re.sub("[^0-9]", "", text))
    
    # List of lines
    with open("log.txt","r") as f:
        log = f.readlines()

    # Cut
    if lines_number >= len(log):
        lines_number = len(log)
    
    # Compose message
    message = "\n".join(log[-lines_number:])
    
    # Chunk and send
    for i in range(0, len(message), settings["max_message_length"]):
        bot.sendMessage(
            chat_id, 
            message[i : i + settings["max_message_length"]], 
            disable_web_page_preview=True
            )

def handle(msg):
    """
    This function is the message handler.
    It checks if the message is a valid command and calls the correct function (according to settings).
    """
    content_type, chat_type, chat_id, date, message_id = telepot.glance(msg, long=True)
    text = msg["text"]
    
    # Construct log message 
    log_message = ("New message - "
        "Chat ID: " + str(chat_id) + ", "
        "Chat type: " + chat_type + ", "
        "Content type: " + content_type + ", "
        "Text: " + text + ", "
        "Date: " + str(date) + ", "
        "Message ID: " + str(message_id)
    )
    
    # Load chats file
    with open("chats.json") as f:
        chats = json.load(f)
    
    # Check if the user is the admin and act accordingly
    if chat_id != chats["admin_chat"]:
        log_message += " - DUMPED."
        bot.sendMessage(chat_id, settings["redirect_message"])
        
    elif content_type != "text":
        log_message += " - DUMPED."
        
    else:
        log_message += " - OK."
        # Call correct function
        for item in settings["allowed_commands"]:
            if item["command"] in text:
                func = item["function"]
                globals()[func](chat_id, text, item)
    
    # Log
    with open("log.txt","a+") as log:
        log.write(str(datetime.datetime.now()) + " - " + log_message + "\n")

def main():
    """
    Main function.
    It initializes the bot and it loads the settings.
    Then it starts the message handler thread.
    Finally it loops forever, launching the update function at a rate defined in the settings.
    """
    
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
    
    # Message handler loop
    MessageLoop(bot, handle).run_as_thread()
    
    # Updates enabled
    global active
    active = True
    
    # Log startup
    with open("log.txt","a+") as log:
        log.write(str(datetime.datetime.now()) + " - Startup.\n")
    
    # Update loop
    while(1):
        if active:
            with open("log.txt","a+") as log:
                log.write(str(datetime.datetime.now()) + " - Update begins.\n")
        
            update()
            
            with open("log.txt","a+") as log:
                log.write(str(datetime.datetime.now()) + " - Update ends.\n")
        
        time.sleep(settings["seconds_between_updates"])

if __name__ == "__main__":
    main()