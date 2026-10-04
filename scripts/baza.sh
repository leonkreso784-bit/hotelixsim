#!/usr/bin/env bash
# Lokalna baza za HotelixSim: zaseban PostgreSQL 16 klaster u korisničkoj mapi, na priključku 5433.
# Ne dira sistemski PostgreSQL servis (5432). Nakon ponovnog pokretanja računala: `bash scripts/baza.sh start`.
# Upotreba: bash scripts/baza.sh start | stop | status
set -euo pipefail

PGBIN="${PGBIN:-/c/Program Files/PostgreSQL/16/bin}"
PGDATA_DIR="${HOTELIXSIM_PGDATA:-C:/Users/$USERNAME/AppData/Local/hotelixsim/pgdata}"
LOG="$(dirname "$PGDATA_DIR")/server.log"
PORT=5433

case "${1:-status}" in
  start)  "$PGBIN/pg_ctl.exe" -D "$PGDATA_DIR" -o "-p $PORT" -l "$LOG" -w start ;;
  stop)   "$PGBIN/pg_ctl.exe" -D "$PGDATA_DIR" -w stop ;;
  status) "$PGBIN/pg_ctl.exe" -D "$PGDATA_DIR" status ;;
  *) echo "upotreba: bash scripts/baza.sh start | stop | status" >&2; exit 2 ;;
esac
