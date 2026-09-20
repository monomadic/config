#!/usr/bin/env python3

import sys
from selenium import webdriver
import os

def main():
    if len(sys.argv) < 2:
        print("Usage: python script.py <url>")
        sys.exit(1)

    url = sys.argv[1]

    # Start the browser
    browser = webdriver.Firefox()

    try:
        # Load the page
        browser.get(url)

        # Wait for the page to load fully
        video = browser.find_element_by_tag_name('video')

        # Extract the video URL
        video_url = video.get_attribute('src')

        # Pass the video URL to yt-dlp
        os.system(f"yt-dlp {video_url}")

    finally:
        browser.quit()

if __name__ == "__main__":
    main()
