# Construction de la plateforme AMAZONE COMP (ADR 0008).
# Le contexte de build est la RACINE DU DÉPÔT (docker-compose.yml s'en charge) :
#   docker compose -f plateforme/docker-compose.yml build
FROM rust:1-slim AS construction
WORKDIR /source
COPY . .
WORKDIR /source/plateforme
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=construction /source/plateforme/target/release/amazone-plateforme /usr/local/bin/
ENV PLATEFORME_ADRESSE=0.0.0.0:8080
EXPOSE 8080
USER nobody
ENTRYPOINT ["amazone-plateforme"]
