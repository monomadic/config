#!/usr/bin/env python
""" In: a firefox cookies.sqlite file
    	default $HOME/Library/Application\ Support/Firefox/Profiles/*/cookies.sqlite
    Out: urls in the file, one per line on stdout

modified https://linuxfreelancer.com/decoding-firefox-cookies-sqlite-cookies-viewer
"""
# google python sqlite3 "delete cookies" ? looks mttiw


import glob
import os
import sqlite3
import sys

__version__ = "2016-01-14 jan  denis-bz-py t-online de"


def Usage():
    print("/.../cookies.sqlite not found")
    sys.exit(1)


def dollarstar(filename):
    """expand $vars, *"""
    filename = os.path.expandvars(filename)  # e.g. $HOME
    names = glob.glob(filename)  # -> [] or [name ...]
    return names[0] if len(names) > 0 else filename  # 2 or more ?


if len(sys.argv) >= 2:
    sqldb = sys.argv[1]
else:
    sqldb = dollarstar(
        "$HOME/Library/Application Support/Firefox/Profiles/*/cookies.sqlite"
    )

if not os.path.isfile(sqldb):
    Usage()

# ...............................................................................
# Bind to the sqlite db and execute sql statements
conn = sqlite3.connect(sqldb)
cur = conn.cursor()
try:
    data = cur.execute("select * from moz_cookies")
except sqlite3.Error:
    print("Error:")
    sys.exit(1)

mydata = data.fetchall()

#  0  id
#  1  baseDomain
#  2  appId
#  3  inBrowserElement
#  4  name
#  5  value
#  6  host
#  7  path
#  8  expiry
#  9  lastAccessed
# 10  creationTime
# 11  isSecure
# 12  isHttpOnly

# urls only, no datetimes --
urls = sorted(set([item[1] for item in mydata]))
for url in urls:
    print(url)
