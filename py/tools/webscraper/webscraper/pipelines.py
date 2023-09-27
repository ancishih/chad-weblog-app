# Define your item pipelines here
#
# Don't forget to add your pipeline to the ITEM_PIPELINES setting
# See: https://docs.scrapy.org/en/latest/topics/item-pipeline.html


# useful for handling different item types with a single interface
from itemadapter import ItemAdapter
import psycopg2
import json
class WebscraperPipeline:
    def __init__(self):
        hostname = '172.17.0.1'
        username = 'postgres'
        password = 'ilove4lice'
        database = 'postgres'

        self.connection = psycopg2.connect(host=hostname,user=username,password=password,dbname=database)
        self.cur = self.connection.cursor()

    def process_item(self, item, spider):
        data = json.dumps(item["data"])
        self.cur.execute("""insert into demo_app.company_income(symbol, earning_calendar, income_statement) values (%s, %s, %s)""",
                         (item["symbol"],item["earning_calendar"],data))
        self.connection.commit()
        return item
    

    def close_spider(self, spider):
        self.cur.close()
        self.connection.close()
