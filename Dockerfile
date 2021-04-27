FROM python:3
WORKDIR /usr/src/app
COPY ./app/requirements.txt ./requirements.txt
RUN pip3 install -r requirements.txt
COPY ./app .
CMD ["bot.py"]
ENTRYPOINT ["python3"]
