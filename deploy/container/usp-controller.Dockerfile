FROM scratch
COPY target/x86_64-unknown-linux-musl/release/usp-controller /usp-controller
USER 65532:65532
EXPOSE 3100
ENTRYPOINT ["/usp-controller"]
