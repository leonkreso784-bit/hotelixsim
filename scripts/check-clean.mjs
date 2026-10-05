// check:api: generirane datoteke moraju biti praćene i nepromijenjene. `git diff --exit-code` ne vidi
// NOVU nepraćenu datoteku (BACKLOG [dnevnik], Task 6 #2 kostura); `git status --porcelain` je vidi.
import { execFileSync } from "node:child_process";

const paths = process.argv.slice(2);
if (paths.length === 0) {
  console.error("upotreba: node scripts/check-clean.mjs <putanja>…");
  process.exit(2);
}
const out = execFileSync("git", ["status", "--porcelain", "--", ...paths], { encoding: "utf8" });
if (out.trim() !== "") {
  console.error(`generirane datoteke nisu commitane:\n${out}`);
  process.exit(1);
}
console.log(`čisto: ${paths.join(", ")}`);
