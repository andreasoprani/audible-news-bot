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
    
    with open("books.json") as f:
        booksSent = json.load(f)
    
    # GET request
    content = requests.get(settings["url"]).content

    # BS data structure
    soup = BeautifulSoup(content, features="lxml")

    # Retrieve books
    productList = soup.findAll("li", "productListItem")
    
    
    global log_update

    # Check date and get info
    for product in reversed(productList):
    
        # Get info
        book_title = product.get("aria-label") #.encode('latin1').decode('utf-8')
        book_author = product.find("li", "authorLabel").find("a").contents[0] #.encode('latin1').decode('utf-8')
        book_narrator = product.find("li", "narratorLabel").find("a").contents[0] #.encode('latin1').decode('utf-8')
        book_runtime = product.find("li", "runtimeLabel").find("span").contents[0].replace("Durata:  ","")
        book_imageURL = product.find("img", "bc-image-inset-border").get("src")
        book_URL = product.find("a", "bc-link").get("href")
        
        # Get date
        dateString = product.find("li", "releaseDateLabel").find("span").contents[0]
        book_date = datetime.date(*[int(s) for s in dateString.split() if s.isdigit()][::-1])

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
        
        if book not in booksSent:
            # Log
            with open("log.txt","a+") as log:
                log.write(str(datetime.datetime.now()) + " - " + str(book) + "\n")
            log_update += str(datetime.datetime.now()) + " - " + str(book) + "\n\n"
            
            # Add to new books list
            booksSent.append(book)
            
            # Update books file
            with open('books.json', 'w') as f:
                json.dump(booksSent, f, indent=4, separators=(',', ': '))
            
            # Send
            sendBookToAll(book)
            
    # Delete older books
    while len(booksSent) > settings["max_books_kept"]:
        booksSent.pop(0)
    with open('books.json', 'w') as f:
        json.dump(booksSent, f, indent=4, separators=(',', ': '))

def start(chat_id, text, command):
    """
    This function is called when the admin uses the /start command.
    It starts updates.
    """
    
    # Activate updates
    with open("chats.json", "r") as f:
        chats = json.load(f)
    chats["active"] = True
    with open("chats.json", "w") as f:
        json.dump(chats, f, indent=4, separators=(',', ': '))
    
    # Log
    with open("log.txt","a+") as log:
        log.write(str(datetime.datetime.now()) + " - Updates activated.\n")
    global log_update
    log_update += str(datetime.datetime.now()) + " - Updates activated.\n\n"

def stop(chat_id, text, command):
    """
    This function is called when the admin uses the /stop command.
    It stops updates.
    """
    
    # De-activate updates
    with open("chats.json", "r") as f:
        chats = json.load(f)
    chats["active"] = False
    with open("chats.json", "w") as f:
        json.dump(chats, f, indent=4, separators=(',', ': '))
    
    # Log
    with open("log.txt","a+") as log:
        log.write(str(datetime.datetime.now()) + " - Updates de-activated.\n")
    global log_update
    log_update += str(datetime.datetime.now()) + " - Updates de-activated.\n\n"

def log(chat_id, text, command):
    """
    This function is called whenever the admin uses the /log command.
    It sends the log of the bot.
    """

    # Number of lines requested
    lines_string = re.sub("[^0-9]", "", text)
    if lines_string == "":
        lines_number = settings["default_log_length"]
    else:
        lines_number = int(lines_string)
    
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
    
    # It tells if the message has to get through
    ok = False
    
    # Load chats file
    with open("chats.json", "r") as f:
        chats = json.load(f)
    
    # Check if the user is the admin and act accordingly
    if chat_id != chats["admin_chat"]:
        log_message += " - DUMPED."
        bot.sendMessage(chat_id, settings["redirect_message"])

    elif content_type != "text":
        log_message += " - DUMPED."
        
    else:
        log_message += " - OK."
        ok = True   
    
    # Log
    with open("log.txt","a+") as log:
        log.write(str(datetime.datetime.now()) + " - " + log_message + "\n")
    global log_update
    log_update += str(datetime.datetime.now()) + " - " + log_message + "\n\n"
    
    # If ok, call correct function
    if ok:
        for item in settings["allowed_commands"]:
            if item["command"] in text:
                func = item["function"]
                globals()[func](chat_id, text, item)

def handleMessages():
    
    # Find last update id
    with open("chats.json", "r") as f:
        last_update = json.load(f)["last_update"]
        
    # Get all messages in queue.
    updates = bot.getUpdates(offset = last_update + 1)
    
    # If there are no updates, return
    if len(updates) == 0:
        return
    
    # Handle every message.
    for update in updates:
        handle(update["message"])
    
    # Update the last update id 
    with open("chats.json", "r") as f:
        chats = json.load(f)
    chats["last_update"] = updates[-1]["update_id"]
    with open("chats.json", "w") as f:
        json.dump(chats, f, indent=4, separators=(',', ': '))

def main():
    """
    Main function.
    It initializes the bot and it loads the settings.
    It handles the messages.
    It launches the update function.
    It sends the final log to the admin.
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
    
    # Initialize log update for the admin
    global log_update
    log_update = "LOG - " + str(datetime.datetime.now()) + "\n\n"
    
    # Log startup
    with open("log.txt","a+") as log:
        log.write(str(datetime.datetime.now()) + " - Startup.\n")
    log_update += str(datetime.datetime.now()) + " - Startup.\n\n"
    
    # Handle messages
    handleMessages()
    
    # Retrieve if updates are enabled
    with open("chats.json", "r") as f:
        chats = json.load(f)
        active = chats["active"]
    
    # Update
    if active:
        with open("log.txt","a+") as log:
            log.write(str(datetime.datetime.now()) + " - Update begins.\n")
        log_update += str(datetime.datetime.now()) + " - Update begins.\n\n"
        
        update()
            
        with open("log.txt","a+") as log:
            log.write(str(datetime.datetime.now()) + " - Update ends.\n")
        log_update += str(datetime.datetime.now()) + " - Update ends.\n\n"
    
    # Send log update to the admin
    with open("chats.json") as f:
        chats = json.load(f)
        
        # Chunk and send
        for i in range(0, len(log_update), settings["max_message_length"]):
            bot.sendMessage(
                chats["admin_chat"], 
                log_update[i : i + settings["max_message_length"]], 
                disable_web_page_preview=True
                )

if __name__ == "__main__":
    main()