build:
	docker build . -t gkmiller/language-helper:latest

run:
	docker run -p 3000:3000 gkmiller/language-helper:latest
