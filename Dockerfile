FROM python:3
WORKDIR /usr/src/app
COPY ./app .
RUN pip3 install -r requirements.txt
CMD ["bot.py"]
ENTRYPOINT ["python3"]
