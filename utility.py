def confrontDate(date1, date2):
        for i in range(0,len(date1)):
                if date1[i] != date2[i]:
                        return False
        return True

def messageBuilder(books, date):
        if len(books) == 0:
                return
            
        output = "Novità del " + dateString(date) + "\n\n"
        
        for book in books:
                output += "Titolo: " + book[0] + "\n"
                output += "Autore: " + book[1] + "\n"
                output += "Narratore: " + book[2] + "\n"
                output += "Durata: " + book[3] + "\n"
                output += "Image URL: " + book[4] + "\n"
                output += "\n"
        
        return output

def dateString(date):
        output = ""
        for el in date:
                if el < 10:
                        output += "0"
                output += str(el)
                if el != date[-1]:
                        output += "-"
        return output
