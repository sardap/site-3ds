FROM devkitpro/devkitarm:20250102

RUN apt-get update -y \
    && apt-get install -y build-essential curl git nodejs npm \
    && curl https://sh.rustup.rs -sSf | bash -s -- -y

RUN mkdir -p /repos
WORKDIR /repos
RUN git clone https://github.com/devkitPro/libctru.git
WORKDIR /repos/libctru/libctru
RUN make install

ENV PATH="/root/.cargo/bin:${PATH}"
ENV PATH="/opt/devkitpro/devkitARM/bin/:${PATH}"

RUN rustup override set nightly \
    && rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu \
    && cargo install cargo-3ds

RUN mkdir /app
WORKDIR /app

RUN apt-get install -y 

COPY ./site/package.json /app/site/package.json
COPY ./site/package-lock.json /app/site/package-lock.json

WORKDIR /app/site
RUN npm install

WORKDIR /app

ENTRYPOINT [ "/bin/bash" ]