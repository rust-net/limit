FROM alpine:latest
COPY target/aarch64-unknown-linux-musl/debug/app /app
ENTRYPOINT ["/app"]