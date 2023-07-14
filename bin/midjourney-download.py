#!/usr/bin/env python3

import requests
import argparse


def download_file(image_url, file_name):
    response = requests.get(image_url, stream=True)
    response.raise_for_status()

    with open(file_name, "wb") as file:
        for chunk in response.iter_content(chunk_size=8192):
            file.write(chunk)


def main():
    parser = argparse.ArgumentParser(description="midjourney-dl")
    parser.add_argument("job-id", type=str, help="job-id")

    args = parser.parse_args()

    image_url = "https://cdn.midjourney.com/{job-id}/0_0.webp"
    file_name = "{job-id}-0_0.webp"

    download_file(args.input, file_name)


if __name__ == "__main__":
    main()
