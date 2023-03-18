# 웹 오목


## UI 폴더
#### 오목 UI 부분 public 폴더에 빌드되어있는 wasm 파일을 사용



## Wasm 폴더
#### 오목 인공지능 Wasm 관련 빌드 후 UI 폴더의 public 폴더로 wasm 복사 필요



## 완료
#### 오목 UI 완료(2인 플레이가능, https://hellotesthello.duckdns.org/gomoku )
#### 싱글스레드 wasm 인공지능 완료.
#### 오목 UI에 wasm 적용. 
#### IOS Out of Memory 관련 문제 해결(wasm memory 최대 크기 256MB로 설정, 기종마다 다름)


## 진행중
#### 도커 적용



## 추가사항
#### 멀티스레드 wasm 인공지능 수정
#### Transposition Table 구현(오목 포지션 점수 연산에 대한 캐시 적용). 



## UI 실행방법
#### 1. docker 실행(macos, windows의 경우 docker desktop 실행, linux는 sudo systemctl start docker)
#### 2. cd ui 
#### 3. docker build . -t ui-con
#### 4. docker run -p 8080:8080 -d ui-con
#### 5. 브라우저에서 http://localhost:8080 접속
