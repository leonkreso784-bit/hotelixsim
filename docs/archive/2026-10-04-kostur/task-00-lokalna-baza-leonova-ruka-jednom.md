### Task 0: Lokalna baza (Leonova ruka, jednom)

Lozinku superkorisnika `postgres` zna samo Leon. Agent ovaj korak **ne radi**, nego ga traži od Leona i čeka.

- [ ] **Step 1: Leon u Claude Code promptu upiše (zamijeni `<lozinka-postgresa>`):**

```text
! "C:/Program Files/PostgreSQL/16/bin/psql.exe" -h localhost -U postgres -c "CREATE ROLE hotelixsim LOGIN PASSWORD 'hotelixsim_lokalno' CREATEDB;" -c "CREATE DATABASE hotelixsim OWNER hotelixsim;"
```

psql će pitati lozinku korisnika `postgres`. `CREATEDB` treba zato što `#[sqlx::test]` za svaki test stvara privremenu bazu.

- [ ] **Step 2: Provjera (agent)**

Run: `PGPASSWORD=hotelixsim_lokalno "/c/Program Files/PostgreSQL/16/bin/psql.exe" -w -h localhost -U hotelixsim -d hotelixsim -c "select current_user;"`
Expected: jedan redak `hotelixsim`.

- [ ] **Step 3: `.env` u korijenu repoa (agent; datoteka je već u `.gitignore`)**

```text
DATABASE_URL=postgres://hotelixsim:hotelixsim_lokalno@localhost:5432/hotelixsim
HOTELIXSIM_ADDR=127.0.0.1:8080
```

Provjera da nije u gitu: `git check-ignore .env` mora ispisati `.env`.

---

