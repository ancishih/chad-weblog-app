from selenium import webdriver


options = webdriver.ChromeOptions()
options.add_argument('--headless')
options.add_argument('--disable-gpu')
options.add_argument('--no-sandbox')
options.add_argument('--disable-dev-shm-usage')
driver = webdriver.Chrome(options=options)

driver.get('http://www.amazon.com/')

title_element = driver.find_element('id','twotabsearchtextbox')
title_element.send_keys('apple macbook')
