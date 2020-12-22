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

RUN wget https://gstreamer.freedesktop.org/src/gstreamer/gstreamer-1.16.3.tar.xz \
    && tar xvfJ gstreamer-1.16.3.tar.xz > /dev/null \
    && cd gstreamer-1.16.3 \
    && ./configure --prefix=/usr \
    && make \
    && make install

RUN wget https://gstreamer.freedesktop.org/src/gst-plugins-base/gst-plugins-base-1.16.3.tar.xz \
    && tar xvfJ gst-plugins-base-1.16.3.tar.xz > /dev/null \
    && cd gst-plugins-base-1.16.3 \
    && ./configure --prefix=/usr \
    && make \
    && make install

RUN git clone https://github.com/libnice/libnice.git --branch 0.1.16 \
    && cd libnice \
    && ./autogen.sh --prefix=/usr --with-gstreamer --enable-static --enable-static-plugins --enable-shared --without-gstreamer-0.10 --disable-gtk-doc \
    && make install

RUN wget https://gstreamer.freedesktop.org/src/gst-plugins-good/gst-plugins-good-1.16.3.tar.xz \
    && tar xvfJ gst-plugins-good-1.16.3.tar.xz > /dev/null \
    && cd gst-plugins-good-1.16.3 \
    && ./configure --prefix=/usr \
    && make \
    && make install

RUN wget https://gstreamer.freedesktop.org/src/gst-plugins-bad/gst-plugins-bad-1.16.3.tar.xz \
    && tar xvfJ gst-plugins-bad-1.16.3.tar.xz > /dev/null \
    && cd gst-plugins-bad-1.16.3 \
    && ./configure --prefix=/usr \
    && make \
    && make install

RUN wget https://gstreamer.freedesktop.org/src/gst-plugins-ugly/gst-plugins-ugly-1.16.3.tar.xz \
    && tar xvfJ gst-plugins-ugly-1.16.3.tar.xz > /dev/null \
    && cd gst-plugins-ugly-1.16.3 \
    && ./configure --prefix=/usr \
    && make \
    && make install

RUN wget https://gstreamer.freedesktop.org/src/gst-rtsp-server/gst-rtsp-server-1.16.3.tar.xz \
    && tar xvfJ gst-rtsp-server-1.16.3.tar.xz > /dev/null \
    && cd gst-rtsp-server-1.16.3 \
    && ./configure --prefix=/usr \
    && make \
    && make install

WORKDIR /app