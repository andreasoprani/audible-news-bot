import json
import requests
from bs4 import BeautifulSoup
import datetime
import utility

with open("bot_settings.json") as f:
    settings = json.load(f)

# Get url from settings
url = settings["url"]

# GET request
content = requests.get(url).content

# BS data structure
soup = BeautifulSoup(content, features="lxml")

# Retrieve books
productList = soup.findAll("li", "productListItem")

# Get today date
today = datetime.datetime.now()

books = []

# Check date and get info
for product in productList:

    # Check date
    dateString = product.find("li", "releaseDateLabel").find("span").contents[0]
    date = [int(s) for s in dateString.split() if s.isdigit()]
    if not utility.confrontDate(date, [today.day, today.month, today.year]):
        break
    
    # Get info
    title = product.get("aria-label")
    author = product.find("li", "authorLabel").find("a").contents[0]
    narrator = product.find("li", "narratorLabel").find("a").contents[0]
    runtime = product.find("li", "runtimeLabel").find("span").contents[0].replace("Durata:  ","")

    books.append([title, author, narrator, runtime])

for book in books:
    print(book)