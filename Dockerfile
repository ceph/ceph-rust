FROM buildpack-deps:focal

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.46.0

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu';; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf';; \
        arm64) rustArch='aarch64-unknown-linux-gnu';; \
        i386) rustArch='i686-unknown-linux-gnu';; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.22.1/${rustArch}/rustup-init"; \
    wget "$url"; \
    wget "$url.sha256"; \
    sed -i 's/target.*/rustup-init/g' rustup-init.sha256; \
    sha256sum -c rustup-init.sha256; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN apt-get update \
    && apt-get install apt-transport-https \
    && wget -q -O- 'https://download.ceph.com/keys/release.asc' | apt-key add - \
    && echo "deb https://download.ceph.com/debian-octopus/ focal main" > /etc/apt/sources.list.d/ceph.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        uuid-runtime \
        ceph-mgr ceph-mon ceph-osd ceph-mds \
        librados-dev libradosstriper-dev

# update crates.io index
RUN cargo search --limit 0

WORKDIR /ceph-rust

COPY micro-osd.sh /
COPY setup-micro-osd.sh /
COPY entrypoint.sh /

CMD /entrypoint.sh
