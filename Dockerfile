# HotelixSim — slika za pilot (Railway). Više faza: Rust poslužitelj + izgrađeni web + mala završna slika.
# Gradi se u CI-ju (GitHub Actions); lokalno Docker nije potreban. Tajne NISU u slici: lozinka stranice,
# DATABASE_URL i ANTHROPIC_API_KEY dolaze kao varijable okoline na Railwayu.

# ---- 1. Rust: poslužitelj ------------------------------------------------------------------------
# Verzija = rust-toolchain.toml (channel 1.98.1).
FROM rust:1.98.1-slim-bookworm AS rust-build
WORKDIR /src
# aws-lc-sys (rustls) prevodi C kod; cmake i pkg-config su sigurnosna rezerva.
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake pkg-config \
 && rm -rf /var/lib/apt/lists/*
# Cijeli Cargo workspace (svi članovi moraju postojati da bi se Cargo.toml razriješio).
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
# prompts/ se ugrađuju u binarnu (include_str! u crates/ai); .sqlx/ zbog SQLX_OFFLINE; migracije su ugrađene
# (sqlx::migrate! u crates/store, već u crates/).
COPY prompts ./prompts
COPY .sqlx ./.sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release -p server --locked

# ---- 2. Node: izgrađeni web ----------------------------------------------------------------------
FROM node:24-bookworm-slim AS web-build
WORKDIR /src
RUN corepack enable
# Prvo samo manifesti (sloj ovisnosti se kešira dok se oni ne promijene).
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml tsconfig.base.json ./
COPY apps/web/package.json ./apps/web/package.json
COPY packages/api-client/package.json ./packages/api-client/package.json
COPY packages/i18n/package.json ./packages/i18n/package.json
RUN corepack pnpm install --frozen-lockfile
# Izvori. packages/* nemaju vlastiti build (main/types pokazuju na src/index.ts); web ih uključuje izravno.
COPY apps/web ./apps/web
COPY packages ./packages
# content/ treba samo testovima koji ga čitaju (?raw) tijekom tsc-a; mali je.
COPY content ./content
RUN corepack pnpm --filter @hotelixsim/web build

# ---- 3. Završna slika ----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
# ca-certificates: izlazni HTTPS (Anthropic API).
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin hotelixsim
WORKDIR /app
COPY --from=rust-build /src/target/release/server /usr/local/bin/server
COPY --from=web-build /src/apps/web/dist /app/web
COPY content/hotel /app/content
USER hotelixsim
# HOTELIXSIM_CONTENT je mapa koju server čita (kod: content::repo_dir() = content/hotel), pa je /app/content = content/hotel.
# Railway daje PORT; ako javna mreža ne pogodi 8080, postavi na Railwayu PORT=8080 (vidi izvještaj, 🟨).
ENV HOTELIXSIM_ADDR=0.0.0.0:8080 \
    HOTELIXSIM_WEB=/app/web \
    HOTELIXSIM_CONTENT=/app/content \
    HOTELIXSIM_PRODUKCIJA=1
EXPOSE 8080
# Migracije pa poslužitelj; exec da SIGTERM stigne do poslužitelja (uredno gašenje).
CMD ["sh", "-c", "server migrate && exec server serve"]
