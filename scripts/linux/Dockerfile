# syntax=docker/dockerfile:1
FROM node:24-bookworm AS node_runtime
FROM rust:1-bookworm AS verification
COPY --from=node_runtime /usr/local/bin/node /usr/local/bin/node
COPY --from=node_runtime /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s /usr/local/lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s /usr/local/lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx
RUN apt-get update && apt-get install -y --no-install-recommends \
    libwebkit2gtk-4.1-dev build-essential pkg-config curl wget file \
    libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*
RUN npm install --global pnpm@10.17.1
ENV CARGO_BUILD_JOBS=2
WORKDIR /workspace
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY . .
RUN mkdir /verification && rustc --version > /verification/toolchain.txt \
    && node --version >> /verification/toolchain.txt \
    && pnpm --version >> /verification/toolchain.txt \
    && cat /etc/os-release >> /verification/toolchain.txt \
    && sha256sum pnpm-lock.yaml src-tauri/Cargo.lock > /verification/locks.sha256
RUN pnpm test > /verification/frontend-tests.txt 2>&1 \
    || (cat /verification/frontend-tests.txt && exit 1)
RUN pnpm build > /verification/frontend-build.txt 2>&1 \
    || (cat /verification/frontend-build.txt && exit 1)
RUN cargo test --manifest-path src-tauri/Cargo.toml --locked --lib \
    > /verification/native-tests.txt 2>&1 \
    || (cat /verification/native-tests.txt && exit 1)
RUN pnpm tauri build --bundles deb -- --locked \
    > /verification/package-build.txt 2>&1 \
    || (cat /verification/package-build.txt && exit 1)
RUN set -e; for package in src-tauri/target/release/bundle/deb/*.deb; do \
      dpkg-deb --info "$package" >> /verification/package-info.txt; \
      dpkg-deb --contents "$package" >> /verification/package-contents.txt; \
      sha256sum "$package" >> /verification/packages.sha256; \
    done
FROM scratch AS artifacts
COPY --from=verification /workspace/src-tauri/target/release/bundle/deb/ /packages/
COPY --from=verification /verification/ /verification/
