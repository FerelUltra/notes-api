FROM rust:1.88

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src
COPY migrations ./migrations

RUN cargo build

EXPOSE 3000

CMD ["cargo", "run", "--bin", "notes_api"]
