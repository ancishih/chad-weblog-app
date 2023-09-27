import scrapy
from scrapy import Selector

class BalanceSheetSpider(scrapy.Spider):
    name = "balance-sheet"
    allowed_domains = ["www.futunn.com"]

    def start_requests(self):
        headers = {
            "cookie":"futunn_lang=en-US;locale=en-us;",
            "Accept-Language":"en-US,en;q=0.7"
        }
        with open('symbol.csv') as file:
            urls = file.readlines()
        for url in urls:
            url = url.strip()
            yield scrapy.Request(url=url,callback = self.parse, headers=headers)

    def parse(self, response):
        
        url = response.url
        
        symbol = url.split('-')[0].split('/')[-1]
        
        date = response.xpath("//div[@class='display-flex date-head']/node()/text()").getall()

        root = response.xpath('//section[@class="data-body"]')
        
        child = root.xpath('node()[contains(@class,"child-item")]')

        data = []

        for x in range(1,len(child)+1):
            node = root.xpath('node()[contains(@class,"child-item")][position()=$n]',n=x)
                           
            child_node = node.xpath('node()').getall()
            if child_node != None:
                _len = len(child_node)
                match _len:
                    case 1:
                        title = node.xpath("node()/@title").get()
                        data.append([title])
                    case 2:
                        subtitle = node.xpath("node()/@title").get()
                        data.append([subtitle,""])
                    case 11:
                        element = []
                        data_container = []
                        for r in range(0,len(child_node)):
                            sel = Selector(text=child_node[r])
                            if r==0:
                                title = sel.xpath(".//@title").get()
                                child_span = sel.xpath(".//span[@class='child-span']/text()").get()
                                if child_span!=None:
                                    title = child_span + " " + title
                                element.append(title)
                            else:
                                val = sel.xpath(".//@title").getall()
                                data_container.append(val)
                        element.append(data_container)
                        data.append(element)

        WebscraperItem = {
            "earning_calendar":date,
            "data":data,
            "symbol":symbol
        }
        yield WebscraperItem