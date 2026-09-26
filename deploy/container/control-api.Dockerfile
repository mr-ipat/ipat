FROM scratch
COPY target/x86_64-unknown-linux-musl/release/control-api /control-api
USER 65532:65532
EXPOSE 3000
ENTRYPOINT ["/control-api"]
