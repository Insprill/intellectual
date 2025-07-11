####################################################################################################
## Builder
####################################################################################################
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /intellectual

COPY . .

# Set environment variables so the build has git info
RUN export $(cat .env | xargs) && cargo build --release

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
