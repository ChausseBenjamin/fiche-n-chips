FROM golang:1.26-alpine AS compile

RUN apk --no-cache add \
  git \
  upx

WORKDIR /build

COPY go.mod go.sum ./
RUN go mod download && go mod verify

COPY internal ./internal/
COPY main.go ./

# RUN go build -o /app -ldflags="-v -w -s" .
RUN go build -o /app .

RUN upx --overlay=strip /app

FROM scratch AS package

COPY --from=compile /app /app

ENTRYPOINT ["/app"]
