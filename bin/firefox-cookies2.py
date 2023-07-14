import glob
import http.cookiejar
import os
import sqlite3


def glob_path(filename):
    """expand $vars, *"""
    filename = os.path.expandvars(filename)  # e.g. $HOME
    names = glob.glob(filename)  # -> [] or [name ...]
    return names[0] if len(names) > 0 else filename  # 2 or more ?


def get_firefox_cookies(domain):
    # Path to the Firefox profile
    cookie_db_path = glob_path(
        "$HOME/Library/Application Support/Firefox/Profiles/*/cookies.sqlite"
    )

    print(cookie_db_path)

    # Connect to the database and run a query for cookies from the specified domain
    conn = sqlite3.connect(cookie_db_path)
    cursor = conn.cursor()
    cursor.execute(
        "SELECT name, value FROM moz_cookies WHERE host LIKE ?", (f"%{domain}%",)
    )

    # Convert the results into a cookie dictionary
    cookies = {}
    for row in cursor.fetchall():
        cookies[row[0]] = row[1]

    return cookies


import requests

# Get cookies
cookies = get_firefox_cookies("midjourney.com")

# Make the request
response = requests.get("http://midjourney.com", cookies=cookies)

# Print the response
print(response.content)
