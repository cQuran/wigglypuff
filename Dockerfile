FROM cquran/webrtc-base:0.1.0 as builder

ADD Cargo.toml .

ADD src src/

RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release --target x86_64-unknown-linux-musl 

RUN ldd /app/target/x86_64-unknown-linux-musl/release/wigglypuff | tr -s '[:blank:]' '\n' | grep '^/' | \
    xargs -I % sh -c 'mkdir -p $(dirname depsz%); cp % depsz%;'

FROM alpine:3.12

COPY --from=builder /app/depsz /
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/wigglypuff /bin/
COPY --from=builder /usr/local/lib /usr/local/lib
COPY --from=builder /usr/lib /usr/lib
COPY --from=builder /lib/libuuid* /lib/

RUN sh -c "ldconfig; echo \$?"
ENTRYPOINT ["/bin/wigglypuff"]