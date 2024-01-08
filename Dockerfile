####################################################################################################
## Builder
####################################################################################################
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev gcc # GCC needed for aarch64 builds

WORKDIR /intellectual

COPY . .

# Figure out what arch we're on
RUN BASE_TARGET=-unknown-linux-musl; \
    case "$(uname -m)" in \
        x86_64) TARGET=x86_64$BASE_TARGET ;; \
        aarch64) TARGET=aarch64$BASE_TARGET ;; \
        *) echo "Unsupported architecture"; exit 1 ;; \
    esac; \
# Build the binary
    cargo build --target ${TARGET} --release; \
    mkdir --parents ./target/docker/; \
    mv ./target/${TARGET}/release/intellectual ./target/docker/intellectual

####################################################################################################
## Final image
####################################################################################################
FROM alpine:latest

# Copy our build
COPY --from=builder /intellectual/target/docker/intellectual /usr/local/bin/intellectual

# Use an unprivileged user
RUN adduser --home /nonexistent --no-create-home --disabled-password intellectual
USER intellectual

# Tell Docker to expose port 8080
EXPOSE 8080/tcp

# Run a healthcheck every 5 minutes
HEALTHCHECK --interval=5m --timeout=5s CMD wget --tries=1 --spider http://localhost:8080 || exit 1

CMD ["intellectual"]
