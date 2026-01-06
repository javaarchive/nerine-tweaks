FROM caddy:2.10.2-builder-alpine AS builder

COPY . ./pkg

RUN xcaddy build \
    --with github.com/ctf-gg/nerine=./pkg/ \
    --with github.com/caddy-dns/cloudflare

FROM caddy:2.10.2-alpine

COPY --from=builder /usr/bin/caddy /usr/bin/caddy