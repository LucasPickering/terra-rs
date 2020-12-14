FROM node:14.2.0-slim AS builder

ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get update && \
    apt-get install -y \
    build-essential \
    curl \
    && \
    rm -rf /var/lib/apt/lists/*

# Install rustup and wasm-pack
RUN curl -sSLf https://sh.rustup.rs | sh -s -- -y --default-toolchain none && \
    curl -sSLf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh

# Copy in sources and build the project
COPY . /app
WORKDIR /app/typescript
RUN npm run build --mode=production

# Copy the built assets into another image
FROM alpine:latest
COPY --from=builder /app/typescript/dist /app/static
