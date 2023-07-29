#!python3

import argparse
import os

import browser_cookie3
import cloudscraper
import requests
from requests.cookies import RequestsCookieJar

# import urllib.request


UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 13.4; rv:109.0) Gecko/20100101 Firefox/115.0"
HEADERS = {"User-Agent": UA}


def download_file_cf(url, filename):
    print("using cloudflare scraper")
    scraper = cloudscraper.create_scraper()
    response = scraper.get(url, headers=HEADERS)
    # response = scraper.get(url)
    response.raise_for_status()
    with open(filename, "wb") as file:
        for chunk in response.iter_content(chunk_size=8192):
            file.write(chunk)


# def download_file_urllib(url, filename):
#     opener = urllib.request.build_opener()
#     opener.addheaders = [("User-agent", UA)]
#     urllib.request.install_opener(opener)
#     urllib.request.urlretrieve(url, filename)

# def download_file(url, filename):
#     jar = browser_cookie3.firefox(domain_name=".midjourney.com")
#     c = RequestsCookieJar()
#
#     for cookie in jar:
#         print("cookie: " + cookie.name)
#         if cookie.name == "__cf_bm" or cookie.name == "cf_clearance":
#             print(cookie)
#             c.set(name=cookie.name, value=cookie.value)
#
#     response = requests.get(url, cookies=c, headers=HEADERS)
#     response.raise_for_status()
#
#     with open(filename, "wb") as file:
#         for chunk in response.iter_content(chunk_size=8192):
#             file.write(chunk)


def scrape_page(url):
    jar = browser_cookie3.load(domain_name=".midjourney.com")
    c = RequestsCookieJar()

    for cookie in jar:
        c.set(name=cookie.name, value=cookie.value)

    response = requests.get(url, cookies=c, headers=HEADERS)
    response.raise_for_status()

    return response.text

    # with open(filename, "wb") as file:
    #     for chunk in response.iter_content(chunk_size=8192):
    #         file.write(chunk)


def main(args):
    image_url = f"https://www.midjourney.com/app/jobs/{args.job_id}/"
    print(scrape_page(image_url))


# for i in range(0, 4):
#     image_url = f"https://cdn.midjourney.com/{args.job_id}/0_{i}.{args.format}"
#     filename = f"{args.job_id}-0_{i}.{args.format}"
#     if os.path.exists(filename):
#         print("File already exists, skipping: " + filename)
#     else:
#         print(f"Requesting {image_url}")
#         download_file_cf(image_url, filename)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="midjourney-dl")
    parser.add_argument("job_id", type=str, help="job id")
    parser.add_argument(
        "-f", "--format", type=str, default="webp", help="jpg, png, webp"
    )
    args = parser.parse_args()
    main(args)
