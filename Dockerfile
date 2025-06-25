FROM alpine:latest
COPY target/x86_64-unknown-linux-musl/debug/app /app
# COPY target/aarch64-unknown-linux-musl/debug/app /app
ENTRYPOINT ["/app"]