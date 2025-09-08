import fs from "fs";
import path from "path";
import { rootPath } from "./utils";
import { AnchorTypeGenerator } from "./codegen";

async function main() {
  const inputDir = rootPath("./scripts/common/interfaces");
  const files = fs.readdirSync(inputDir);

  const tsFiles = files.filter(
    (file) =>
      file.endsWith(".ts") &&
      file !== "index.ts" &&
      !file.endsWith(".anchor.ts")
  );

  if (tsFiles.length === 0) {
    console.log("No valid .ts files found for code generation.");
    return;
  }

  for (const file of tsFiles) {
    const inputFile = path.join(inputDir, file);
    const outputFile = inputFile.replace(/\.ts$/, ".anchor.ts");

    try {
      const generator = new AnchorTypeGenerator(inputFile);
      const generatedCode = generator.generate();

      fs.writeFileSync(outputFile, generatedCode);
      console.log(`Generated anchor types and converters for ${file}`);
    } catch (error) {
      console.error(`Error generating types for ${file}:`, error);
    }
  }
}

main();
