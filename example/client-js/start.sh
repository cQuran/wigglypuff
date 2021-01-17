docker stop wigglypuffjs
docker build -t wigglypuff-js .
docker run --rm --name wigglypuffjs -p 6080:80 wigglypuff-js