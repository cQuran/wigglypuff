FROM rust:1.47-slim-buster

RUN apt update \
    && apt install -y --no-install-recommends \
        git wget build-essential ca-certificates \
        python3 python3-pip python3-setuptools \
        python3-wheel pkg-config libmount-dev flex \
        bison openssl cmake libglib2.0 libpixman-1-0 libcairo2-dev \
        libffi-dev libnice-dev libopus-dev libpcre3-dev \
        libsrtp2-dev libssl-dev libtool libvpx-dev libx264-dev \
        autoconf automake gnutls-dev gtk-doc-tools \
    && pip3 install meson ninja

WORKDIR /opt

ENV LD_LIBRARY_PATH /usr

RUN wget https://gstreamer.freedesktop.org/src/gstreamer/gstreamer-1.18.2.tar.xz \
    && tar xvfJ gstreamer-1.18.2.tar.xz > /dev/null \
    && cd gstreamer-1.18.2 \
    && meson build \
    && ninja -C build install

RUN wget https://gstreamer.freedesktop.org/src/gst-plugins-base/gst-plugins-base-1.18.2.tar.xz \
    && tar xvfJ gst-plugins-base-1.18.2.tar.xz > /dev/null \
    && cd gst-plugins-base-1.18.2 \
    && meson build \
    && ninja -C build install

RUN git clone https://github.com/libnice/libnice.git --branch 0.1.18 \
    && cd libnice \
    && meson build \
    && ninja -C build install

RUN wget https://gstreamer.freedesktop.org/src/gst-plugins-good/gst-plugins-good-1.18.2.tar.xz \
    && tar xvfJ gst-plugins-good-1.18.2.tar.xz > /dev/null \
    && cd gst-plugins-good-1.18.2 \
    && meson build \
    && ninja -C build install

RUN wget https://gstreamer.freedesktop.org/src/gst-plugins-bad/gst-plugins-bad-1.18.2.tar.xz \
    && tar xvfJ gst-plugins-bad-1.18.2.tar.xz > /dev/null \
    && cd gst-plugins-bad-1.18.2 \
    && meson build \
    && ninja -C build install

RUN wget https://gstreamer.freedesktop.org/src/gst-plugins-ugly/gst-plugins-ugly-1.18.2.tar.xz \
    && tar xvfJ gst-plugins-ugly-1.18.2.tar.xz > /dev/null \
    && cd gst-plugins-ugly-1.18.2 \
    && meson build \
    && ninja -C build install

RUN wget https://gstreamer.freedesktop.org/src/gst-rtsp-server/gst-rtsp-server-1.18.2.tar.xz \
    && tar xvfJ gst-rtsp-server-1.18.2.tar.xz > /dev/null \
    && cd gst-rtsp-server-1.18.2 \
    && meson build \
    && ninja -C build install

RUN ldconfig -v

RUN apt update && apt install libnss3-tools -y \
    && wget https://github.com/FiloSottile/mkcert/releases/download/v1.4.3/mkcert-v1.4.3-linux-amd64 \
    && mv mkcert-v1.4.3-linux-amd64 mkcert \
    && chmod +x mkcert \
    && cp mkcert /usr/local/bin/ \
    && mkcert -install

WORKDIR /app

RUN mkcert 127.0.0.1 localhost

ADD Cargo.toml .

ADD src src/

# RUN RUSTFLAGS="-C target-cpu=native" cargo install --path .

# RUN ldd /usr/local/cargo/bin/analytics-store-service | tr -s '[:blank:]' '\n' | grep '^/' | \
#     xargs -I % sh -c 'mkdir -p $(dirname deps%); cp % deps%;'

# FROM scratch

# COPY --from=builder /app/deps /
# COPY --from=builder /usr/local/cargo/bin/wigglypuff /bin/

# ENTRYPOINT ["/bin/wigglypuff"]