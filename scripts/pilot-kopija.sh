#!/usr/bin/env bash
# Kopije baze pilota (task-16, dio 7): pg_dump s Railwaya na Leonovo računalo + proba vraćanja u zasebnu lokalnu bazu.
# Upotreba: bash scripts/pilot-kopija.sh kopija | proba <datoteka.dump> | usporedi
# PILOT_DATABASE_URL (adresa baze pilota s lozinkom) je samo varijabla okoline — nikad u git, nikad u ispis.
# Neobavezno: PGBIN (mapa PG programa), PILOT_KOPIJE_MAPA (izlaz; zadano Documents/hotelixsim-kopije/pilot),
# PROBA_HOST/PROBA_PORT/PROBA_USER/PROBA_PASSWORD (lokalni klaster; zadano localhost/5433/hotelixsim/hotelixsim_lokalno).
set -euo pipefail

PGBIN="${PGBIN:-/c/Program Files/PostgreSQL/16/bin}"
MAPA="${PILOT_KOPIJE_MAPA:-C:/Users/$USERNAME/Documents/hotelixsim-kopije/pilot}"
PROBA_DB="hotelixsim_proba_kopije"   # UVIJEK ova; nikad hotelixsim ni hotelixsim_agenti
PROBA_HOST="${PROBA_HOST:-localhost}"
PROBA_PORT="${PROBA_PORT:-5433}"
PROBA_USER="${PROBA_USER:-hotelixsim}"
PROBA_PASSWORD="${PROBA_PASSWORD:-hotelixsim_lokalno}"

greska() { echo "GREŠKA: $*" >&2; exit 1; }

treba_izvor() {
  [ -n "${PILOT_DATABASE_URL:-}" ] || greska "nedostaje PILOT_DATABASE_URL (adresa baze pilota; postavi je u terminalu, ne u git)."
}

verzija_programa() { "$PGBIN/pg_dump.exe" --version | sed -E 's/.* ([0-9]+)(\.[0-9]+)*.*/\1/'; }

# pg_dump mora biti >= verzija poslužitelja, inače odbija raditi.
provjeri_verzije() {
  local srv alat
  srv="$("$PGBIN/psql.exe" -X -At -d "$PILOT_DATABASE_URL" -c 'show server_version_num' 2>/dev/null)" \
    || greska "ne mogu se spojiti na PILOT_DATABASE_URL (adresa/mreža/lozinka)."
  srv=$((srv / 10000))
  alat="$(verzija_programa)"
  if [ "$alat" -lt "$srv" ]; then
    greska "pg_dump je verzije $alat, a poslužitelj pilota $srv — kopija ne radi. Instaliraj PostgreSQL $srv alate i postavi PGBIN."
  fi
  echo "verzije: pg_dump $alat, poslužitelj $srv (u redu)"
}

brojevi() { # $1 = adresa baze; ispisuje games i events
  local g e
  g="$("$PGBIN/psql.exe" -X -At -d "$1" -c 'select count(*) from games')"
  e="$("$PGBIN/psql.exe" -X -At -d "$1" -c 'select count(*) from events')"
  echo "games=$g events=$e"
}

lokalna_adresa() { # $1 = ime baze
  echo "postgresql://$PROBA_USER:$PROBA_PASSWORD@$PROBA_HOST:$PROBA_PORT/$1"
}

case "${1:-}" in
  kopija)
    treba_izvor
    provjeri_verzije
    mkdir -p "$MAPA"
    izlaz="$MAPA/$(date +%Y-%m-%d_%H%M%S).dump"
    "$PGBIN/pg_dump.exe" -Fc --no-owner --no-privileges -d "$PILOT_DATABASE_URL" -f "$izlaz"
    echo "kopija: $izlaz ($(wc -c < "$izlaz") B)"
    ;;
  proba)
    dump="${2:-}"
    [ -n "$dump" ] || greska "upotreba: bash scripts/pilot-kopija.sh proba <datoteka.dump>"
    [ -f "$dump" ] || greska "datoteka ne postoji: $dump"
    export PGPASSWORD="$PROBA_PASSWORD"
    "$PGBIN/dropdb.exe" -h "$PROBA_HOST" -p "$PROBA_PORT" -U "$PROBA_USER" --if-exists "$PROBA_DB"
    "$PGBIN/createdb.exe" -h "$PROBA_HOST" -p "$PROBA_PORT" -U "$PROBA_USER" "$PROBA_DB"
    # obriši probnu bazu i kad vraćanje ne uspije
    trap '"$PGBIN/dropdb.exe" -h "$PROBA_HOST" -p "$PROBA_PORT" -U "$PROBA_USER" --if-exists "$PROBA_DB" && echo "probna baza obrisana"' EXIT
    "$PGBIN/pg_restore.exe" -h "$PROBA_HOST" -p "$PROBA_PORT" -U "$PROBA_USER" -d "$PROBA_DB" --no-owner --no-privileges --exit-on-error "$dump"
    echo "iz kopije: $(brojevi "$(lokalna_adresa "$PROBA_DB")")"
    ;;
  usporedi)
    treba_izvor
    echo "izvor:     $(brojevi "$PILOT_DATABASE_URL")"
    ;;
  *) echo "upotreba: bash scripts/pilot-kopija.sh kopija | proba <datoteka.dump> | usporedi" >&2; exit 2 ;;
esac
