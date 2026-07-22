build:
	docker build . -t gkmiller/language-helper:latest

run:
	docker run --init --rm -p 3000:3000 gkmiller/language-helper:latest
