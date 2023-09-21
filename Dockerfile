FROM debian:testing

RUN apt update \
 && apt -y install \
    adduser \
    cargo \
    git \
    libssl-dev \
    pkg-config
# && apt -y dist-upgrade \

ARG UID
RUN adduser --uid $UID --quiet satoshi
USER satoshi
RUN mkdir -p /home/satoshi/teslatte
WORKDIR /home/satoshi/teslatte

RUN git clone https://github.com/gak/teslatte.git .

RUN cargo update -p time@0.3.28 --precise 0.3.23 || true \
 && cargo update -p time-macros@0.2.14 --precise 0.2.10 || true \
 && cargo update -p anstream@0.5.0 --precise  0.4.0 || true \
 && cargo update -p clap_builder@4.4.2 --precise  4.3.24 || true \
 && cargo update -p clap@4.4.4 --precise  4.3.24 || true \
 && cargo update -p clap_lex@0.5.1 --precise  0.5.0 || true \
 && cargo update -p anstyle@1.0.3 --precise  1.0.2 || true

RUN cargo build --bin teslatte

