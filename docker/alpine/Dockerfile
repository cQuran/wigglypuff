FROM cquran/webrtc-base:0.1.0-alpine as builder

RUN USER=root cargo new --bin wigglypuff
WORKDIR /app/wigglypuff
ADD ./Cargo.lock ./Cargo.lock 
ADD ./Cargo.toml ./Cargo.toml  

RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release --target x86_64-unknown-linux-musl 

ADD src src/

RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release --target x86_64-unknown-linux-musl 

RUN ldd /app/wigglypuff/target/x86_64-unknown-linux-musl/release/wigglypuff | tr -s '[:blank:]' '\n' | grep '^/' | \
    xargs -I % sh -c 'mkdir -p $(dirname depsz%); cp % depsz%;'

FROM alpine:3.12

COPY --from=builder /app/wigglypuff/depsz /
COPY --from=builder /app/wigglypuff/target/x86_64-unknown-linux-musl/release/wigglypuff /bin/
COPY --from=builder /usr/local/lib /usr/local/lib
COPY --from=builder /usr/lib /usr/lib
COPY --from=builder /lib/libuuid* /lib/
ADD static /bin/static

RUN sh -c "ldconfig; echo \$?"

WORKDIR /bin

ENTRYPOINT ["./wigglypuff"]