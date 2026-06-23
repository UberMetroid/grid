# Stage 1: Build the Frontend (Yew WebAssembly)
FROM rust:1.96-alpine as frontend-builder
RUN apk add --no-cache musl-dev wget tar
WORKDIR /usr/src/app

# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown
RUN wget -qO- "https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-musl.tar.gz" | tar -xzf- -C /usr/local/bin

COPY Cargo.toml Cargo.lock ./
COPY backend/ ./backend/
COPY frontend/ ./frontend/
WORKDIR /usr/src/app/frontend
RUN trunk build --release

# Stage 2: Build the Backend
FROM rust:1.96-alpine as backend-builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY backend/ ./backend/
COPY frontend/ ./frontend/
# Compile backend binary
RUN cargo build --release --bin backend

# Stage 3: Final package
FROM alpine:3.18
WORKDIR /usr/src/app

# Install runtime dependencies
RUN apk add --no-cache wget libc6-compat

ENV PORT=4405
ENV NODE_ENV=production

COPY --from=backend-builder /usr/src/app/target/release/backend ./rustkan
COPY --from=frontend-builder /usr/src/app/frontend/dist ./frontend/dist

RUN chown -R 99:100 /usr/src/app

# Run as Unraid nobody:users
USER 99:100

EXPOSE 4405

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s CMD wget -qO- http://localhost:4405/health || exit 1

CMD ["./rustkan"]