#!python3

import argparse
import os

from bs4 import BeautifulSoup


def clean_html(html):
    soup = BeautifulSoup(html, "html.parser")

    # Remove empty tags
    for tag in soup.find_all():
        if not tag.text.strip():
            tag.extract()

    # Remove <style> tags
    for style_tag in soup.find_all("style"):
        style_tag.decompose()

    # Remove <script> tags
    for style_tag in soup.find_all("script"):
        style_tag.decompose()

    # Remove style attributes
    for tag in soup.find_all(True):
        tag.attrs = {key: value for key, value in tag.attrs.items() if key != "style"}

    return str(soup)


def main(args):
    with open(args.input, "r") as file:
        code = file.read()
        print(clean_html(code))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="")
    parser.add_argument("-i", "--input", type=str, help="HTML Document")
    args = parser.parse_args()
    main(args)
