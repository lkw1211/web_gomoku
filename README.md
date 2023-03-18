# 웹 오목

## UI 실행방법
#### 1. docker 실행(macos, windows의 경우 docker desktop 실행, linux는 sudo systemctl start docker)
#### 2. cd ui 
#### 3. docker build . -t ui-con
#### 4. docker run -p 8080:8080 -d ui-con
#### 5. 브라우저에서 http://localhost:8080 접속
