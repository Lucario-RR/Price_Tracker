import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const [, , targetRel, templateRel] = process.argv;
const target = path.isAbsolute(targetRel)
  ? targetRel
  : path.resolve(__dirname, "..", targetRel);
const tmplPath = path.isAbsolute(templateRel)
  ? templateRel
  : path.resolve(__dirname, templateRel);
let source = fs.readFileSync(target, "utf8");
const insert = fs.readFileSync(tmplPath, "utf8");
const start = source.indexOf("<template>");
const end = source.lastIndexOf("</template>") + "</template>".length;
if (start === -1 || end < start) {
  throw new Error("template markers not found: " + target);
}
source = source.slice(0, start) + insert + source.slice(end);
fs.writeFileSync(target, source);
console.log("OK", target);
