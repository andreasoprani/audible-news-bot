# Standard libraries
import time
import json
import datetime
from collections import OrderedDict

# Bot related libraries
import telepot
from telepot.loop import MessageLoop

# HTML parsing related libraries
import requests
from bs4 import BeautifulSoup


def sendBook(chat_id, book):
    
    # Build the message
    message = ""
    for key in settings["attribute_names"].keys():
        message += settings["attribute_names"][key] + ": " + str(book[key]) + "\n"
    
    # Send the book
    bot.sendPhoto(chat_id, book["imageURL"], caption=message)

def sendBookToAll(book):
    # Retrieve chats
    with open("chats.json") as f:
        chats = json.load(f)
    
    # Send book to every chat
    for chat in chats:
        sendBook(chat["chat_id"], book)

def update():
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

        # Create book dict
        book = {
            "title": book_title,
            "author": book_author,
            "narrator": book_narrator,
            "runtime": book_runtime,
            "date": str(book_date),
            "imageURL": book_imageURL
        }
        
        # Add new books to the books list and send it to everyone
        if book not in todayBooks["books"]:
            todayBooks["books"].append(book)
            sendBookToAll(book)
    
    # Update today_books file
    with open('today_books.json', 'w') as f:
        json.dump(todayBooks, f, indent=4, separators=(',', ': '))

def start(chat_id, command):
    with open("chats.json") as f:
        chats = json.load(f)
    
    chat = next((item for item in chats if item["chat_id"]==chat_id), False)
    
    if not chat:
        chats.append({
            "chat_id": chat_id,
            "active": True
        })
    else:
        chat["active"] = True

    with open('chats.json', 'w') as f:
        json.dump(chats, f, indent=4, separators=(',', ': '))
    
    bot.sendMessage(chat_id, command["message"])

def stop(chat_id, command):
    with open("chats.json") as f:
        chats = json.load(f)
    
    chat = next((item for item in chats if item["chat_id"]==chat_id), False)
    
    if not chat:
        chats.append({
            "chat_id": chat_id,
            "active": False
        })
    else:
        chat["active"] = False

    with open('chats.json', 'w') as f:
        json.dump(chats, f, indent=4, separators=(',', ': '))
        
    bot.sendMessage(chat_id, command["message"])
    
def help(chat_id, command):
    help_message = command["message"]
    
    for item in settings["allowed_commands"]:
        help_message += item["command"] + ": " + item["description"] + "\n"
        
    bot.sendMessage(chat_id, help_message)

def today(chat_id, command):
    # Initial today books message
    today_books_message = command["message"]
    bot.sendMessage(chat_id, today_books_message)
    
    # Load today books
    with open("today_books.json") as f:
        todayBooks = json.load(f)
    
    # Send every book in today books
    for book in todayBooks["books"]:
        sendBook(chat_id, book)

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
    
    # Message handler loop
    MessageLoop(bot, handle).run_as_thread()
    print("Listening...")
    
    # Update loop
    while(1):
        update()
        print("Update - " + str(datetime.datetime.now()) + "\n")
        time.sleep(settings["seconds_between_updates"])

if __name__ == "__main__":
    main()