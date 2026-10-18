####################################################################################################
## Builder
####################################################################################################
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /intellectual

# Build dependencies in their own layer so it stays cached until Cargo.toml or Cargo.lock change
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

COPY . .

# Set environment variables so the build has git info
# main.rs is touched because COPY keeps its old mtime, which cargo would treat as already built
RUN export $(cat .env | xargs) && touch src/main.rs && cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM alpine:latest

# Copy our build
COPY --from=builder /intellectual/target/release/intellectual /usr/local/bin/intellectual

# Use an unprivileged user
RUN adduser --home /nonexistent --no-create-home --disabled-password intellectual
USER intellectual

# Tell Docker to expose port 8080
EXPOSE 8080/tcp

# Run a healthcheck every 5 minutes
HEALTHCHECK --interval=1m --timeout=3s --start-period=3s CMD wget --spider -q http://localhost:8080 || exit 1

CMD ["intellectual"]
