#!/usr/bin/env node
// Pregled banke ljudi za Leona: iz svih serija (content/hotel/v2/ljudi/{radnici,gosti}/serija-*.json) gradi jednu HTML
// stranicu s filtrima i sažetkom raspodjela. Nema ovisnosti. Upotreba:
//   node scripts/ljudi-pregled.mjs [izlaz.html] [korijen-sadržaja] [--avatari <avatari.mjs>]
// Zadano: .superpowers/sdd/banka-ljudi-k1/pregled-svi.html i content/hotel/v2/ljudi.
// Avatari (neobavezno): bundle gradi `node scripts/ljudi-avatari-build.mjs <avatari.mjs>` (vite iz apps/web, izvori iz
// ../hotelixsim-izgled). Bez --avatari stranica radi kao prije, samo bez stupca "lik".
// 🟨 Privremena pravila hinta za avatar (K5 daje točna): gender = rod (z→f, m→m); age_group: dob < 30 young, 30–54 adult,
// ≥ 55 senior; origin: radnik lokalno/regija → regional, inozemstvo → foreign; gost zemlja ∈ {HR,BA,RS,ME,MK,SI} → regional,
// inače foreign. Sjeme imena = `${ime} ${prezime}`.
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const argv = process.argv.slice(2);
const ai = argv.indexOf("--avatari");
const avatariPut = ai >= 0 ? resolve(argv[ai + 1]) : null;
if (ai >= 0) argv.splice(ai, 2);
const out = resolve(argv[0] ?? ".superpowers/sdd/banka-ljudi-k1/pregled-svi.html");
const root = resolve(argv[1] ?? "content/hotel/v2/ljudi");
const svgFor = avatariPut ? (await import(pathToFileURL(avatariPut).href)).svgFor : null;
const REGIJA = new Set(["HR", "BA", "RS", "ME", "MK", "SI"]);
function hint(z) {
  const origin =
    z.vrsta === "radnik"
      ? z.podrijetlo === "inozemstvo"
        ? "foreign"
        : "regional"
      : REGIJA.has(z.zemlja)
        ? "regional"
        : "foreign";
  return {
    gender: z.rod === "z" ? "f" : "m",
    age_group: z.dob < 30 ? "young" : z.dob < 55 ? "adult" : "senior",
    origin,
  };
}
let avatarBr = 0;
const avatar = (z) => {
  const n = ++avatarBr;
  // useId daje isti id u svakom SVG-u; za jednu stranicu moraju biti jedinstveni
  return svgFor(`${z.ime} ${z.prezime}`, hint(z)).replace(/lik-_R_[^"')]*?_/g, `lik-${n}-`);
};

function serije(vrsta) {
  const dir = join(root, vrsta);
  return readdirSync(dir)
    .filter((f) => /^serija-\d{3}\.json$/.test(f))
    .sort()
    .map((f) => JSON.parse(readFileSync(join(dir, f), "utf8")));
}

const radnici = serije("radnici").flatMap((s) =>
  s.zapisi.map((z) => ({ ...z, vrsta: "radnik", serija: s.serija })),
);
const gosti = serije("gosti").flatMap((s) =>
  s.zapisi.map((z) => ({ ...z, vrsta: "gost", serija: s.serija })),
);
const svi = [...radnici, ...gosti];

const esc = (s) =>
  String(s ?? "").replace(
    /[&<>"']/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c],
  );
const jezici = (j) => (Array.isArray(j) ? j.map(([k, r]) => `${k} ${r}`).join(", ") : "");
const dobRazred = (d) => (d < 25 ? "18-24" : d < 35 ? "25-34" : d < 50 ? "35-49" : "50-65");

function broji(lista, kljuc) {
  const m = new Map();
  for (const z of lista) {
    const k = kljuc(z);
    m.set(k, (m.get(k) ?? 0) + 1);
  }
  return [...m.entries()].sort((a, b) => b[1] - a[1]);
}
function sazetak(naslov, lista, kljuc) {
  const n = lista.length || 1;
  const redci = broji(lista, kljuc)
    .map(
      ([k, v]) =>
        `<tr><td>${esc(k)}</td><td class="n">${v}</td><td class="n">${((100 * v) / n).toFixed(0)} %</td></tr>`,
    )
    .join("");
  return `<details><summary>${esc(naslov)}</summary><table class="mini"><tbody>${redci}</tbody></table></details>`;
}

const opcije = (lista, kljuc) =>
  [...new Set(lista.map(kljuc).filter(Boolean))]
    .sort((a, b) => String(a).localeCompare(String(b), "hr"))
    .map((v) => `<option value="${esc(v)}">${esc(v)}</option>`)
    .join("");

const redak = (z) => {
  const polja = [
    z.id,
    z.vrsta,
    z.serija,
    `${z.ime} ${z.prezime}`,
    z.rod === "z" ? "ž" : "m",
    z.dob,
    z.vrsta === "radnik"
      ? `${z.uloga} · ${z.razina} · ${z.godine_iskustva} g.`
      : `${z.segment ?? ""}${z.zanimanje ? " · " + z.zanimanje : ""}`,
    z.vrsta === "radnik" ? `${z.podrijetlo} · ${z.zemlja}` : z.zemlja,
    z.mjesto,
    jezici(z.jezici),
    z.opis,
  ];
  const data = `data-vrsta="${esc(z.vrsta)}" data-serija="${z.serija}" data-rod="${esc(z.rod)}" data-zemlja="${esc(z.zemlja)}" data-uloga="${esc(z.uloga ?? z.segment ?? "")}" data-dob="${dobRazred(z.dob)}"`;
  return `<tr ${data}>${svgFor ? `<td class="av">${avatar(z)}</td>` : ""}${polja.map((p, i) => `<td class="${i === 10 ? "opis" : i === 5 || i === 2 ? "n" : ""}">${esc(p)}</td>`).join("")}</tr>`;
};

const html = `<!doctype html>
<html lang="hr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Banka ljudi — svi</title>
<style>
:root { --bg:#fbfaf7; --fg:#1d1d1b; --muted:#6b6a64; --line:#e3e0d8; --head:#f1eee6; --accent:#2f5d8a; --chip:#e8eef5; }
@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { --bg:#161615; --fg:#ecebe6; --muted:#a3a19a; --line:#34332f; --head:#22211f; --accent:#8db4dc; --chip:#24303c; } }
:root[data-theme="dark"] { --bg:#161615; --fg:#ecebe6; --muted:#a3a19a; --line:#34332f; --head:#22211f; --accent:#8db4dc; --chip:#24303c; }
* { box-sizing: border-box; }
body { margin:0; background:var(--bg); color:var(--fg); font:15px/1.45 system-ui, -apple-system, "Segoe UI", sans-serif; }
main { max-width: 1500px; margin: 0 auto; padding: 16px; }
h1 { font-size: 22px; margin: 0 0 4px; }
.muted { color: var(--muted); }
.filtri { display:flex; flex-wrap:wrap; gap:8px; margin:12px 0; align-items:center; }
.filtri select, .filtri input { padding:6px 8px; border:1px solid var(--line); border-radius:6px; background:var(--bg); color:var(--fg); font:inherit; }
.filtri input { min-width: 220px; }
.broj { font-weight:600; }
table { width:100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align:left; vertical-align:top; padding:6px 8px; border-bottom:1px solid var(--line); }
th { position: sticky; top:0; background: var(--head); cursor:pointer; white-space:nowrap; }
td.n { text-align:right; white-space:nowrap; }
td.av { padding:2px 6px; } td.av svg { display:block; }
td.opis { min-width: 320px; max-width: 560px; }
tr[hidden] { display:none; }
.sazetci { display:flex; flex-wrap:wrap; gap:12px; margin:8px 0 16px; }
details { border:1px solid var(--line); border-radius:8px; padding:6px 10px; background:var(--head); }
summary { cursor:pointer; font-weight:600; }
table.mini { width:auto; font-size:13px; margin-top:6px; }
table.mini td { padding:2px 8px; border:0; }
.tag { display:inline-block; padding:1px 6px; border-radius:10px; background:var(--chip); font-size:12px; }
@media (max-width: 700px) { td.opis { min-width: 200px; } }
</style>
</head>
<body>
<main>
<h1>Banka ljudi — svi zapisi</h1>
<p class="muted">Radnika: ${radnici.length} · gostiju: ${gosti.length} · serije: ${[...new Set(svi.map((z) => z.serija))].join(", ")}. Klik na naslov stupca sortira. Izvor: <code>${esc(root)}</code>.</p>
<div class="sazetci">
${sazetak("Radnici po ulozi", radnici, (z) => z.uloga)}
${sazetak("Radnici po razini", radnici, (z) => z.razina)}
${sazetak("Radnici po podrijetlu", radnici, (z) => z.podrijetlo)}
${sazetak("Radnici po zemlji", radnici, (z) => z.zemlja)}
${sazetak("Radnici po dobi", radnici, (z) => dobRazred(z.dob))}
${sazetak("Radnici po rodu", radnici, (z) => (z.rod === "z" ? "ž" : "m"))}
${sazetak("Gosti po zemlji", gosti, (z) => z.zemlja)}
${sazetak("Gosti po segmentu", gosti, (z) => z.segment ?? "—")}
${sazetak("Gosti po dobi", gosti, (z) => dobRazred(z.dob))}
${sazetak("Gosti po rodu", gosti, (z) => (z.rod === "z" ? "ž" : "m"))}
</div>
<div class="filtri">
  <select id="f-vrsta"><option value="">vrsta: sve</option><option value="radnik">radnici</option><option value="gost">gosti</option></select>
  <select id="f-serija"><option value="">serija: sve</option>${opcije(svi, (z) => String(z.serija))}</select>
  <select id="f-uloga"><option value="">uloga / segment: sve</option>${opcije(svi, (z) => z.uloga ?? z.segment ?? "")}</select>
  <select id="f-zemlja"><option value="">zemlja: sve</option>${opcije(svi, (z) => z.zemlja)}</select>
  <select id="f-rod"><option value="">rod: sve</option><option value="z">ž</option><option value="m">m</option></select>
  <select id="f-dob"><option value="">dob: sve</option><option>18-24</option><option>25-34</option><option>35-49</option><option>50-65</option></select>
  <input id="f-tekst" type="search" placeholder="pretraži ime, mjesto, opis…">
  <span>prikazano: <span class="broj" id="broj">${svi.length}</span></span>
</div>
<table id="t">
<thead><tr>${svgFor ? "<th>lik</th>" : ""}<th>ID</th><th>vrsta</th><th>serija</th><th>ime</th><th>rod</th><th>dob</th><th>uloga · razina · iskustvo / segment · zanimanje</th><th>podrijetlo · zemlja</th><th>mjesto</th><th>jezici</th><th>opis</th></tr></thead>
<tbody>
${svi.map(redak).join("\n")}
</tbody>
</table>
</main>
<script>
(function () {
  const t = document.getElementById("t");
  const rows = Array.from(t.tBodies[0].rows);
  const f = (id) => document.getElementById(id);
  const ids = ["f-vrsta", "f-serija", "f-uloga", "f-zemlja", "f-rod", "f-dob"];
  function primijeni() {
    const q = f("f-tekst").value.trim().toLowerCase();
    let n = 0;
    for (const r of rows) {
      let ok = true;
      for (const id of ids) {
        const v = f(id).value;
        if (v && r.dataset[id.slice(2)] !== v) { ok = false; break; }
      }
      if (ok && q && !r.textContent.toLowerCase().includes(q)) ok = false;
      r.hidden = !ok;
      if (ok) n++;
    }
    f("broj").textContent = String(n);
  }
  for (const id of ids) f(id).addEventListener("change", primijeni);
  f("f-tekst").addEventListener("input", primijeni);
  let smjer = 1;
  Array.from(t.tHead.rows[0].cells).forEach((th, i) => {
    th.addEventListener("click", () => {
      smjer = -smjer;
      const sorted = rows.slice().sort((a, b) => {
        const x = a.cells[i].textContent, y = b.cells[i].textContent;
        const nx = Number(x), ny = Number(y);
        const c = !Number.isNaN(nx) && !Number.isNaN(ny) && x !== "" && y !== "" ? nx - ny : x.localeCompare(y, "hr");
        return c * smjer;
      });
      for (const r of sorted) t.tBodies[0].appendChild(r);
    });
  });
})();
</script>
</body>
</html>
`;

mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, html, "utf8");
console.log(`pregled: ${out} (radnika ${radnici.length}, gostiju ${gosti.length})`);
