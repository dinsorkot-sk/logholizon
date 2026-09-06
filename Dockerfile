# syntax=docker/dockerfile:1

# ---------- Stage 1: Rust core ----------
FROM rust:1.85-slim AS core-build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY packages/core/Cargo.toml packages/core/Cargo.toml
COPY packages/cli/Cargo.toml packages/cli/Cargo.toml
COPY migrations ./migrations
COPY packages/core/src ./packages/core/src
COPY packages/cli/src ./packages/cli/src
RUN cargo build --release -p logholizon-core

# ---------- Stage 2: Nuxt app ----------
FROM node:22-slim AS app-build
WORKDIR /build
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml turbo.json ./
COPY packages/app/package.json packages/app/package.json
RUN pnpm install --frozen-lockfile
COPY packages/app ./packages/app
RUN pnpm --dir packages/app run build

# ---------- Stage 3: Runtime ----------
FROM node:22-slim AS runtime
WORKDIR /app
ENV NODE_ENV=production
ENV CORE_URL=http://127.0.0.1:8787
ENV CORE_DATABASE_URL=sqlite:///data/core.db
ENV CORE_HOST=0.0.0.0
ENV CORE_PORT=8787

# Rust core binary
COPY --from=core-build /build/target/release/logholizon-core /usr/local/bin/logholizon-core

# Nuxt server output
COPY --from=app-build /build/packages/app/.output ./app/.output

# Data volume for SQLite
VOLUME /data

EXPOSE 3000 8787

# Start both processes: core on 8787, Nuxt on 3000
CMD ["sh", "-c", "logholizon-core & node /app/.output/server/index.mjs"]