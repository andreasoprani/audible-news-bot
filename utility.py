import requests
from bs4 import BeautifulSoup
from datetime import date

def getNewBooks(url):

        # GET request
        content = requests.get(url).content

        # BS data structure
        soup = BeautifulSoup(content, features="lxml")

        # Retrieve books
        productList = soup.findAll("li", "productListItem")

        # Get today date
        today = date.today()

        books = []

        # Check date and get info
        for product in productList:

                # Check date
                dateString = product.find("li", "releaseDateLabel").find("span").contents[0]
                book_date = date(*[int(s) for s in dateString.split() if s.isdigit()][::-1])
                if book_date != today:
                        break
    
                # Get info
                book_title = product.get("aria-label") #.encode('latin1').decode('utf-8')
                book_author = product.find("li", "authorLabel").find("a").contents[0] #.encode('latin1').decode('utf-8')
                book_narrator = product.find("li", "narratorLabel").find("a").contents[0] #.encode('latin1').decode('utf-8')
                book_runtime = product.find("li", "runtimeLabel").find("span").contents[0].replace("Durata:  ","")
                book_imageURL = product.find("img", "bc-image-inset-border").get("src")

                books.append({
                        "title": book_title,
                        "author": book_author,
                        "narrator": book_narrator,
                        "runtime": book_runtime,
                        "imageURL": book_imageURL
                })

        return books


def messageBuilder(books):
        if len(books) == 0:
                return
            
        output = ""
            
        for book in books:
                output += "Titolo: " + book["title"] + "\n"
                output += "Autore: " + book["author"] + "\n"
                output += "Narratore: " + book["narrator"] + "\n"
                output += "Durata: " + book["runtime"] + "\n"
                output += "URL copertina: " + book["imageURL"] + "\n"
                output += "\n"
        
        return output
