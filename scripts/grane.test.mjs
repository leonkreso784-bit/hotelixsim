import assert from "node:assert/strict";
import { test } from "node:test";
import { procijeni } from "./grane.mjs";

const SADA = Date.parse("2026-10-15T12:00:00Z");

test("spojena grana nije stara ni kad je davno otvorena", () => {
  const [r] = procijeni([{ ime: "model-v2/sloj1-t1", ispred: 0, iza: 40, prviNespojen: null }], SADA);
  assert.equal(r.stanje, "spojena");
});
test("nespojen rad stariji od 24 h je STARA", () => {
  const [r] = procijeni([{ ime: "popravak/a2", ispred: 3, iza: 0, prviNespojen: "2026-10-14T10:00:00Z" }], SADA);
  assert.equal(r.stanje, "STARA");
});
test("nespojen rad mlađi od 24 h je u radu", () => {
  const [r] = procijeni([{ ime: "popravak/a3", ispred: 1, iza: 0, prviNespojen: "2026-10-15T09:00:00Z" }], SADA);
  assert.equal(r.stanje, "u radu");
});
test("točno 24 h nije starije od 24 h pa je u radu", () => {
  const [r] = procijeni([{ ime: "popravak/a4", ispred: 2, iza: 1, prviNespojen: "2026-10-14T12:00:00Z" }], SADA);
  assert.equal(r.stanje, "u radu");
});
test("grana na samoj integraciji (ispred 0, iza 0) je spojena", () => {
  const [r] = procijeni([{ ime: "popravak/a5", ispred: 0, iza: 0, prviNespojen: null }], SADA);
  assert.equal(r.stanje, "spojena");
});
