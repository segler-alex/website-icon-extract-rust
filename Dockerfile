FROM rust:1
ADD . /root
WORKDIR /root
RUN cargo install

CMD [ "website-icon-extract" ]
