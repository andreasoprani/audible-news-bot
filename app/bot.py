# Standard libraries
import os
import time
import json
import datetime
from collections import OrderedDict
import re
import schedule

# Bot related libraries
import telepot
from telepot.loop import MessageLoop

# HTML parsing related libraries
import requests
from bs4 import BeautifulSoup


def safe_send_message(chat_id, text, **kwargs):
    while True:
        try:
            bot.sendMessage(chat_id, text, **kwargs)
        except telepot.exception.TooManyRequestsError as e:
            retry_after = e.json['parameters']['retry_after']
            update_log(f"TooManyRequestsError, {retry_after} seconds.")
            time.sleep(retry_after + 1)
            continue
        break

def update_log(append_string):
    log_message = f"{datetime.datetime.now()} - {append_string}\n"
    with open("data/log.txt","a+") as log:
        log.write(log_message)
    global log_update
    log_update += log_message + "\n"

def send_chunked_message(message):
    for i in range(0, len(message), settings["max_message_length"]):
        safe_send_message(
            int(os.environ.get("ADMIN_CHAT")), 
            message[i : i + settings["max_message_length"]], 
            disable_web_page_preview=True
        )

def send_book_to_channel(book):
    """Send a book to the channel chat."""
    
    rows = [
        f"{value}: {str(book[key])}" for (key, value) in settings["attribute_names"].items()
    ]
        
    url = settings["url_header"] + book["URL"]
    rows.append(f'[{settings["book_url_message"]}]({url})')
    
    message = "\n".join(rows)
    
    safe_send_message(
        int(os.environ.get("CHANNEL_CHAT")), 
        message, 
        parse_mode="Markdown"
    )   

def update():
    """
    This function is used to check for update in the Audible catalog.
    It checks if there are new books at the url given in the settings and it sends the latest books 
    to all the active chats.
    """
    
    with open("data/books.json") as f:
        books_sent = json.load(f)
    
    # GET request
    content = requests.get(settings["url"]).content

    # BS data structure
    soup = BeautifulSoup(content, features="lxml")

    # Retrieve books
    product_list = soup.findAll("li", "productListItem")

    # Check date and get info
    for product in reversed(product_list):
    
        # Get info
        book_title = product.get("aria-label") #.encode('latin1').decode('utf-8')
        
        book_author = product.find("li", "authorLabel")
        if book_author is None:
            book_author = "-"
        else: 
            book_author = book_author.find("a").contents[0] #.encode('latin1').decode('utf-8')
            
        book_narrator = product.find("li", "narratorLabel")
        if book_narrator is None:
            book_narrator = "-"
        else: 
            book_narrator = book_narrator.find("a").contents[0] #.encode('latin1').decode('utf-8')
    
        book_runtime = product.find("li", "runtimeLabel")
        if book_runtime is None:
            book_runtime = "-"
        else:
            book_runtime = book_runtime.find("span").contents[0].replace("Durata:  ","")
        
        book_URL = product.find("a", "bc-link").get("href")
        
        # Get date
        date_string = product.find("li", "releaseDateLabel").find("span").contents[0]
        book_date = datetime.datetime.strptime(re.search(r'\d{2}/\d{2}/\d{4}', date_string).group(), "%d/%m/%Y").date()

        # Create book dict
        book = {
            "title": book_title,
            "author": book_author,
            "narrator": book_narrator,
            "runtime": book_runtime,
            "date": str(book_date),
            "URL": book_URL
        }
        
        is_book_present = (
            len(list(filter(
                lambda b: (
                    b["title"] == book["title"] and 
                    b["author"] == book["author"] and 
                    b["narrator"] == book["narrator"] and 
                    b["runtime"] == book["runtime"] and 
                    b["date"] == book["date"]
                ), books_sent
            ))) > 0
        )
        
        if is_book_present:
            continue

        update_log(str(book))
        
        # Add to new books list
        books_sent.append(book)
        
        # Update books file
        with open('data/books.json', 'w') as f:
            json.dump(books_sent, f, indent=4, separators=(',', ': '))
        
        # Send
        send_book_to_channel(book)
            
    # Delete older books
    while len(books_sent) > settings["max_books_kept"]:
        books_sent.pop(0)
    with open('data/books.json', 'w') as f:
        json.dump(books_sent, f, indent=4, separators=(',', ': '))

def start(chat_id, text, command):
    """
    This function is called when the admin uses the /start command.
    It starts updates.
    """
    
    # Activate updates
    with open("data/bot_state.json", "r+") as f:
        bot_state = json.load(f)
        bot_state["active"] = True
        f.seek(0)
        json.dump(bot_state, f, indent=4, separators=(',', ': '))
        f.truncate()
    
    update_log("Updates activated.")

def stop(chat_id, text, command):
    """
    This function is called when the admin uses the /stop command.
    It stops updates.
    """
    
    # De-activate updates
    with open("data/bot_state.json", "r+") as f:
        bot_state = json.load(f)
        bot_state["active"] = False
        f.seek(0)
        json.dump(bot_state, f, indent=4, separators=(',', ': '))
        f.truncate()
    
    update_log("Updates de-activated.")

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
    with open("data/log.txt","r") as f:
        log = f.readlines()

    # Cut
    if lines_number >= len(log):
        lines_number = len(log)
    
    # Compose message
    message = "\n".join(log[-lines_number:])
    
    # send chunked
    send_chunked_message(message)

def handle(msg):
    """
    This function is the message handler.
    It checks if the message is a valid command and calls the correct function (according to settings).
    """
    content_type, chat_type, chat_id, date, message_id = telepot.glance(msg, long=True)
    text = msg.get('text', "no text")
    
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
    
    # Check if the user is the admin and act accordingly
    if chat_id != int(os.environ.get("ADMIN_CHAT")):
        log_message += " - DUMPED."
        try: # if the bot was stopped, sending a message will fail.
            safe_send_message(chat_id, settings["redirect_message"])
        except Exception as e:
            log_message += " - " + str(e)
    elif content_type != "text":
        log_message += " - DUMPED."
    else:
        log_message += " - OK."
        ok = True   
    
    update_log(log_message)
    
    # If ok, call correct function
    if ok:
        for item in settings["allowed_commands"]:
            if item["command"] in text:
                func = item["function"]
                globals()[func](int(os.environ.get("ADMIN_CHAT")), text, item)

def handle_messages():
    
    # Find last update id
    with open("data/bot_state.json", "r") as f:
        last_update = json.load(f)["last_update"]
        
    # Get all messages in queue.
    updates = bot.getUpdates(offset = last_update + 1)
    
    # If there are no updates, return
    if not len(updates):
        return
    
    # Handle every message.
    for update in updates:
        handle(update["message"])
    
    # Update the last update id 
    with open("data/bot_state.json", "r+") as f:
        bot_state = json.load(f)
        bot_state["last_update"] = updates[-1]["update_id"]
        f.seek(0)
        json.dump(bot_state, f, indent=4, separators=(',', ': '))
        f.truncate()

def main():
    """
    Main function.
    It initializes the bot and it loads the settings.
    It handles the messages.
    It launches the update function.
    It sends the final log to the admin.
    """
    
    # Create the bot.
    global bot
    try:
        bot = telepot.Bot(os.environ.get("TOKEN"))
    except Exception as e:
        update_log("EXCEPTION: " + str(e))

    # Get settings from settings file
    global settings
    try:
        with open("data/bot_settings.json", "r") as f:
            settings = json.load(f)
    except FileNotFoundError:
        print("Settings file not found.")
        return
    
    # Initialize log update for the admin
    global log_update
    log_update = "LOG - " + str(datetime.datetime.now()) + "\n\n"
    
    update_log("Startup.")
    
    # The message handler takes care of all the messages arrived to the bot while sleeping.
    try:
        handle_messages()
    except Exception as e:
        update_log("EXCEPTION: " + str(e))
    
    # Retrieve if updates are enabled
    with open("data/bot_state.json", "r") as f:
        bot_state = json.load(f)
        active = bot_state["active"]
    
    # Update
    if active:
        update_log("Update begins.")
        try:
            update()
        except Exception as e:
            update_log("EXCEPTION: " + str(e))
        update_log("Update ends.")
        
    # send chunked
    send_chunked_message(log_update)

if __name__ == "__main__":
    
    minutes = os.environ.get("MINUTES_INTERVAL")
    hours = os.environ.get("HOURS_INTERVAL")
    days = os.environ.get("DAYS_INTERVAL")
    if minutes is not None:
        schedule.every(int(minutes)).minutes.do(main)
    elif hours is not None:
        schedule.every(int(hours)).hours.do(main)
    elif days is not None:
        schedule.every(int(days)).days.do(main)
    else:
        schedule.every(1).hours.do(main)

    while True:
        schedule.run_pending()
        time.sleep(60)
