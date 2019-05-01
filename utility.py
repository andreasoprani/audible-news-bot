def confrontDate(date1, date2):
    for i in range(0,len(date1)):
        if date1[i] != date2[i]:
            return False
    return True