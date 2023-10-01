FROM --platform=linux/amd64 python:3.11.5-alpine3.18
    # command: tail -f /dev/null
WORKDIR /usr/app
# COPY install.txt .
RUN apk update && apk upgrade
RUN apk add cmake gcc libpq-dev libffi-dev python3-dev
RUN pip install --upgrade pip
# RUN pip install -r install.txt
RUN pip install selenium scrapy ipython psycopg2-binary scrapy-fake-useragent
EXPOSE 4000
CMD ["tail", "-f", "/dev/null"]
# ENTRYPOINT [ "bash" ]