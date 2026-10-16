#!/usr/bin/env node
// Gradi samostalan ES modul `svgFor(ime, hint) => string` (SVG avatara) za scripts/ljudi-pregled.mjs --avatari.
// Izvor su avatar izvori iz stabla izgleda (look.ts s hintom), react/vite dolaze iz apps/web. Bez novih ovisnosti u repou.
//   node scripts/ljudi-avatari-build.mjs <izlaz.mjs> [stablo-izgleda] [apps-web]
// Zadano: stablo-izgleda = ../hotelixsim-izgled, apps-web = apps/web. Vite se uvozi iz apps-web/node_modules.
import { mkdirSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const out = resolve(process.argv[2] ?? "avatari.mjs");
const izgled = resolve(process.argv[3] ?? "../hotelixsim-izgled");
const web = resolve(process.argv[4] ?? "apps/web");
const av = join(izgled, "apps/web/src/avatar").replaceAll("\\", "/");
const nm = join(web, "node_modules").replaceAll("\\", "/");

const entryDir = join(dirname(out), "avatari-entry");
mkdirSync(entryDir, { recursive: true });
const entry = join(entryDir, "entry.tsx");
writeFileSync(
  entry,
  `import { renderToStaticMarkup } from "react-dom/server";
import { Avatar } from "${av}/Avatar";
import { lookFor } from "${av}/look";
export function svgFor(name, hint, size = 56) {
  return renderToStaticMarkup(<Avatar look={lookFor(name, hint)} size={size} />);
}
`,
);

const vite = await import(pathToFileURL(join(nm, "vite/dist/node/index.js")).href);
await vite.build({
  configFile: false,
  root: entryDir,
  logLevel: "warn",
  resolve: {
    alias: [
      { find: /^react-dom(\/.*)?$/, replacement: `${nm}/react-dom$1` },
      { find: /^react(\/.*)?$/, replacement: `${nm}/react$1` },
    ],
  },
  ssr: { noExternal: true },
  build: {
    ssr: entry,
    outDir: dirname(out),
    emptyOutDir: false,
    minify: false,
    rollupOptions: { output: { format: "es", entryFileNames: basename(out) } },
  },
});
console.log(`avatari: ${out}`);
