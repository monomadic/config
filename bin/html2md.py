#!python3

import argparse

import html2text


def convert_html_to_markdown(html_file_path):
    with open(html_file_path, "r", encoding="utf-8") as html_file:
        html_content = html_file.read()
        markdown_content = html2text.html2text(html_content)
        with open("output.md", "w", encoding="utf-8") as md_file:
            md_file.write(markdown_content)


def main():
    parser = argparse.ArgumentParser(description="Convert html file to markdown")
    parser.add_argument("input", type=str, help="html file")

    args = parser.parse_args()

    convert_html_to_markdown(args.input)


if __name__ == "__main__":
    main()
