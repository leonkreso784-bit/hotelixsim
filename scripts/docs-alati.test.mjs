// Test zajedničkih alata dokumentacije: slug, prepoznavanje ograda, prepisivanje poveznica.
import assert from "node:assert/strict";
import { test } from "node:test";
import { prepisiRelativnePoveznice, slug, uOgradi } from "./docs-alati.mjs";

test("slug: primjer iz dodatka (6 riječi)", () => {
  assert.equal(slug("Leonove odluke za put B: ljudi kao u Simsima"), "leonove-odluke-za-put-b-ljudi");
});

test("slug: dijakritika i dž/đ", () => {
  assert.equal(slug("Čaša ćaća Đurđa šuma žaba"), "casa-caca-durda-suma-zaba");
  assert.equal(slug("ČĆĐŠŽ"), "ccdsz");
  assert.equal(slug("Café naïve"), "cafe-naive");
});

test("slug: granica broja riječi", () => {
  assert.equal(slug("a b c d e f g h"), "a-b-c-d-e-f");
  assert.equal(slug("a b c d e f g h", 3), "a-b-c");
});

test("slug: interpunkcija, bez vodeće/prateće crtice, prazan ulaz", () => {
  assert.equal(slug("  --Hello, svijet!!  "), "hello-svijet");
  assert.equal(slug("ADR-015 — ime"), "adr-015-ime");
  assert.equal(slug(""), "");
  assert.equal(slug("???"), "");
  assert.match(slug("Neki naslov (v2) / dio 3"), /^[a-z0-9]+(-[a-z0-9]+)*$/);
});

test("uOgradi: označi ograde ``` zajedno s retcima ograde", () => {
  assert.deepEqual(uOgradi(["a", "```js", "b", "```", "c"]), [false, true, true, true, false]);
});

test("uOgradi: ```` se ne zatvara s ```", () => {
  const r = ["````md", "```", "x", "```", "y", "````", "z"];
  assert.deepEqual(uOgradi(r), [true, true, true, true, true, true, false]);
});

test("uOgradi: ~~~ ograda, i ~ ne zatvara ```", () => {
  assert.deepEqual(uOgradi(["~~~", "a", "```", "b", "~~~", "c"]), [true, true, true, true, true, false]);
});

test("uOgradi: zatvaranje duljom ogradom, ne kraćom; ne smije imati tekst iza", () => {
  assert.deepEqual(uOgradi(["```", "a", "``` x", "b", "````", "c"]), [true, true, true, true, true, false]);
  assert.deepEqual(uOgradi(["````", "a", "```", "b"]), [true, true, true, true]);
});

test("uOgradi: do 3 razmaka uvlake je ograda, 4 nije; prateći razmaci pri zatvaranju", () => {
  assert.deepEqual(uOgradi(["   ```", "a", "```   ", "b"]), [true, true, true, false]);
  assert.deepEqual(uOgradi(["    ```", "a"]), [false, false]);
});

test("uOgradi: neotvorena ograda traje do kraja", () => {
  assert.deepEqual(uOgradi(["a", "```", "b", "c"]), [false, true, true, true]);
});

test("prepisiRelativnePoveznice: relativna poveznica i slika prolaze kroz fn", () => {
  const t = 'Vidi [A](x.md) i ![s](slike/a.png) te [B](y.md "naslov").\n';
  assert.equal(
    prepisiRelativnePoveznice(t, (c) => `../${c}`),
    'Vidi [A](../x.md) i ![s](../slike/a.png) te [B](../y.md "naslov").\n',
  );
});

test("prepisiRelativnePoveznice: http, #sidro, mailto, /apsolutni netaknuti", () => {
  const t = "[a](http://x.hr) [b](https://x.hr/y) [c](#b) [d](mailto:a@b.hr) [e](/abs.md) [f](tel:123)";
  let pozivi = 0;
  const out = prepisiRelativnePoveznice(t, (c) => {
    pozivi++;
    return `../${c}`;
  });
  assert.equal(out, t);
  assert.equal(pozivi, 0);
});

test("prepisiRelativnePoveznice: sidro uz putanju je dio cilja", () => {
  const viđeno = [];
  prepisiRelativnePoveznice("[a](x.md#y)", (c) => {
    viđeno.push(c);
    return c;
  });
  assert.deepEqual(viđeno, ["x.md#y"]);
});

test("prepisiRelativnePoveznice: poveznica u ogradi i inline kodu netaknuta", () => {
  const t = "[a](x.md)\n```md\n[b](y.md)\n```\n`[c](z.md)` i [d](w.md)\n";
  assert.equal(
    prepisiRelativnePoveznice(t, (c) => `../${c}`),
    "[a](../x.md)\n```md\n[b](y.md)\n```\n`[c](z.md)` i [d](../w.md)\n",
  );
});

test("prepisiRelativnePoveznice: ograda ```` s ``` unutra štiti sadržaj", () => {
  const t = "````md\n```\n[b](y.md)\n```\n````\n[a](x.md)\n";
  assert.equal(
    prepisiRelativnePoveznice(t, (c) => `../${c}`),
    "````md\n```\n[b](y.md)\n```\n````\n[a](../x.md)\n",
  );
});

test("prepisiRelativnePoveznice: CRLF ostaje CRLF, bajt-točno", () => {
  const t = "[a](x.md)\r\n\r\nTekst\r\n[b](y.md)\r\n";
  const out = prepisiRelativnePoveznice(t, (c) => `../${c}`);
  assert.equal(out, "[a](../x.md)\r\n\r\nTekst\r\n[b](../y.md)\r\n");
});

test("prepisiRelativnePoveznice: ../ pa slice(3) vraća izvornik", () => {
  const t = 'Dio [X](../X.md#a) i [Y](y.md) ![i](a/b.png)\n```\n[n](n.md)\n```\n`[k](k.md)`\r\n[Z](z.md "t")';
  const gore = prepisiRelativnePoveznice(t, (c) => `../${c}`);
  assert.notEqual(gore, t);
  assert.equal(
    prepisiRelativnePoveznice(gore, (c) => c.slice(3)),
    t,
  );
});
