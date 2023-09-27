# Define here the models for your scraped items
#
# See documentation in:
# https://docs.scrapy.org/en/latest/topics/items.html

import scrapy


class WebscraperItem(scrapy.Item):
    
    earning_calendar = scrapy.Field()
    data = scrapy.Field()
    symbol = scrapy.Field()
