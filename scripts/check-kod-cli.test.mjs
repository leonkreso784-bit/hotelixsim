// Testovi same naredbe `node scripts/check-kod.mjs` (ne samo funkcije provjeri):
// brana ne smije šutjeti, ni preko poveznice, ni brojati agentska radna stabla.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

const SKRIPTA = path.resolve(import.meta.dirname, "check-kod.mjs");
const RED = (n) => "x\n".repeat(n);

function uMapi(datoteke) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "check-kod-cli-"));
  for (const [ime, sadrzaj] of Object.entries(datoteke)) {
    fs.mkdirSync(path.dirname(path.join(dir, ime)), { recursive: true });
    fs.writeFileSync(path.join(dir, ime), sadrzaj);
  }
  return dir;
}
function pokreni(skripta, korijen) {
  const env = { ...process.env, CHECK_KOD_ROOT: korijen };
  const r = spawnSync(process.execPath, [skripta], { env, encoding: "utf8" });
  return { status: r.status, out: r.stdout + r.stderr };
}
function ocisti(...mape) {
  for (const m of mape) fs.rmSync(m, { recursive: true, force: true });
}

test("naredba: datoteka od 1001 redak daje izlaz 1 i poruku", () => {
  const d = uMapi({ "a/velik.rs": RED(1001) });
  try {
    const r = pokreni(SKRIPTA, d);
    assert.equal(r.status, 1, r.out);
    assert.match(r.out, /PREKO 1000 REDAKA/);
  } finally {
    ocisti(d);
  }
});

test("naredba: čista mapa daje izlaz 0 i ✅", () => {
  const d = uMapi({ "a/ok.rs": RED(10) });
  try {
    const r = pokreni(SKRIPTA, d);
    assert.equal(r.status, 0, r.out);
    assert.match(r.out, /✅/);
  } finally {
    ocisti(d);
  }
});

test("naredba pokrenuta kroz poveznicu na scripts/ ne šuti", (t) => {
  const d = uMapi({ "a/velik.rs": RED(1001) });
  const veza = path.join(os.tmpdir(), `check-kod-veza-${process.pid}`);
  try {
    fs.symlinkSync(path.dirname(SKRIPTA), veza, "junction");
  } catch {
    ocisti(d);
    t.skip("poveznica nije moguća na ovom stroju");
    return;
  }
  try {
    const r = pokreni(path.join(veza, "check-kod.mjs"), d);
    assert.equal(r.status, 1, r.out);
    assert.match(r.out, /PREKO 1000 REDAKA/);
  } finally {
    fs.rmSync(veza, { recursive: true, force: true });
    ocisti(d);
  }
});

test("naredba: .claude/worktrees (agentske kopije) se ne broji", () => {
  const d = uMapi({ ".claude/worktrees/x/crates/api/src/schema.rs": RED(1687), "ok.rs": RED(5) });
  try {
    const r = pokreni(SKRIPTA, d);
    assert.equal(r.status, 0, r.out);
    assert.doesNotMatch(r.out, /worktrees/);
  } finally {
    ocisti(d);
  }
});
