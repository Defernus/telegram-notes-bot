FROM rust:1.72.0

WORKDIR /app

COPY . .
RUN cargo install --bin --path ./bot

CMD ["bot"]
