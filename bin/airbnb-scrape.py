import requests
import json
import time
import random
from bs4 import BeautifulSoup
from datetime import datetime, timedelta
import sqlite3

class AirbnbScraper:
    def __init__(self, location, check_in, check_out):
        self.base_url = "https://www.airbnb.com/api/v2/explore_tabs"
        self.location = location
        self.check_in = check_in
        self.check_out = check_out
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        })
        self.db_conn = sqlite3.connect('airbnb_data.db')
        self.create_table()

    def create_table(self):
        cursor = self.db_conn.cursor()
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS listings (
            id INTEGER PRIMARY KEY,
            name TEXT,
            price REAL,
            rating REAL,
            availability TEXT,
            amenities TEXT,
            url TEXT
        )
        ''')
        self.db_conn.commit()

    def get_listings(self, price_min=500, price_max=10000, offset=0):
        params = {
            "query": self.location,
            "checkin": self.check_in,
            "checkout": self.check_out,
            "price_min": price_min,
            "price_max": price_max,
            "room_types[]": "Entire home/apt",
            "offset": offset,
            "items_per_grid": 50,
        }

        response = self.session.get(self.base_url, params=params)
        data = json.loads(response.text)

        return data['explore_tabs'][0]['sections'][0]['listings']

    def scrape_listing_details(self, listing_id):
        url = f"https://www.airbnb.com/rooms/{listing_id}"
        response = self.session.get(url)
        soup = BeautifulSoup(response.text, 'html.parser')

        amenities = [amenity.text for amenity in soup.find_all('div', {'class': '_vzrbjl'})]
        availability = soup.find('div', {'class': '_cvkwaj'}).text if soup.find('div', {'class': '_cvkwaj'}) else "Not available"

        return amenities, availability

    def save_to_db(self, listing):
        cursor = self.db_conn.cursor()
        cursor.execute('''
        INSERT OR REPLACE INTO listings (id, name, price, rating, availability, amenities, url)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ''', (
            listing['id'],
            listing['name'],
            listing['price']['price'],
            listing['avg_rating'],
            listing['availability'],
            json.dumps(listing['amenities']),
            f"https://www.airbnb.com/rooms/{listing['id']}"
        ))
        self.db_conn.commit()

    def scrape(self, max_listings=100):
        offset = 0
        listings_scraped = 0

        while listings_scraped < max_listings:
            listings = self.get_listings(offset=offset)

            for listing in listings:
                if listings_scraped >= max_listings:
                    break

                try:
                    amenities, availability = self.scrape_listing_details(listing['listing']['id'])
                    listing['amenities'] = amenities
                    listing['availability'] = availability
                    self.save_to_db(listing['listing'])
                    listings_scraped += 1
                    print(f"Scraped {listings_scraped} listings")
                except Exception as e:
                    print(f"Error scraping listing {listing['listing']['id']}: {str(e)}")

                # Add a random delay between requests
                time.sleep(random.uniform(2, 5))

            offset += len(listings)

            # Add a longer delay between pagination requests
            time.sleep(random.uniform(5, 10))

    def close(self):
        self.db_conn.close()

if __name__ == "__main__":
    location = "New York, NY"
    check_in = (datetime.now() + timedelta(days=30)).strftime("%Y-%m-%d")
    check_out = (datetime.now() + timedelta(days=37)).strftime("%Y-%m-%d")

    scraper = AirbnbScraper(location, check_in, check_out)
    scraper.scrape(max_listings=50)
    scraper.close()
