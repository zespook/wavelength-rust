FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ffmpeg nodejs xvfb chromium \
	&& rm -rf /var/lib/apt/lists/*
COPY . .
ENV CHROME_PATH=/usr/bin/chromium
EXPOSE 8787
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8787"]