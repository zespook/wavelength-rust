FROM rust:bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --bin wavelength-backend

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/wavelength-backend /usr/local/bin/wavelength-backend
EXPOSE 5000
CMD ["wavelength-backend"]
