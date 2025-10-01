import * as fs from "fs";
import * as path from "path";
import ts from "typescript";
import { rootPath } from "../utils";
import { l } from "../../common/utils";

interface Config {
  src: { directory: string; exclude: string[] }[];
  dist: string;
}

interface TypeInfo {
  name: string;
  members: ts.PropertySignature[];
}

interface Pair {
  intName: string;
  codamaName: string;
  intMembers: ts.PropertySignature[];
  codamaMembers: ts.PropertySignature[];
  category: "types" | "accounts" | "instructions";
}

const primitiveMap: { [key: string]: string } = {
  number: "TUint8",
  Uint8: "TUint8",
  Uint16: "TUint16",
  Uint32: "TUint32",
  Uint64: "TUint64",
  Uint128: "TUint128",
  Address: "TAddress",
  String16: "TString16",
  String32: "TString32",
  String64: "TString64",
  String4096: "TString4096",
  BitField: "TBitField",
};

function getInterfaces(
  sourceFile: ts.SourceFile,
  filePath: string,
): Map<string, ts.PropertySignature[]> {
  const interfaces = new Map<string, ts.PropertySignature[]>();
  ts.forEachChild(sourceFile, (node) => {
    if (
      ts.isInterfaceDeclaration(node) &&
      ts.isIdentifier(node.name) &&
      node.name.text.startsWith("I")
    ) {
      const members = node.members.filter(
        ts.isPropertySignature,
      ) as ts.PropertySignature[];
      interfaces.set(node.name.text, members);
      l(`Found interface ${node.name.text} in ${filePath}`);
    }
  });
  return interfaces;
}

function getTypeFromFile(filePath: string): TypeInfo[] {
  const content = fs.readFileSync(filePath, "utf8");
  const sourceFile = ts.createSourceFile(
    filePath,
    content,
    ts.ScriptTarget.Latest,
    true,
  );
  const types: TypeInfo[] = [];

  ts.forEachChild(sourceFile, (node) => {
    if (ts.isTypeAliasDeclaration(node) && ts.isIdentifier(node.name)) {
      const members = ts.isTypeLiteralNode(node.type)
        ? (node.type.members.filter(
            ts.isPropertySignature,
          ) as ts.PropertySignature[])
        : [];
      types.push({ name: node.name.text, members });
      l(
        `Found type ${node.name.text} in ${filePath} with ${members.length} members`,
      );
    }
  });

  if (types.length === 0) {
    l(`No type aliases found in ${filePath}`);
  }
  return types;
}

function main() {
  const configPath = rootPath("./src/backend/services/path.json");
  if (!fs.existsSync(configPath)) {
    console.error(`Config file ${configPath} not found`);
    return;
  }
  const config: Config = JSON.parse(fs.readFileSync(configPath, "utf8"));

  const codamaRoot = rootPath(config.dist).replace(/codegen$/, "codama");
  const codamaMap = new Map<string, ts.PropertySignature[]>();

  l(`Scanning Codama root: ${codamaRoot}`);
  ["types", "accounts", "instructions"].forEach((cat) => {
    const dir = path.join(codamaRoot, cat);
    if (fs.existsSync(dir)) {
      const files = fs.readdirSync(dir).filter((f) => f.endsWith(".ts"));
      l(`Found ${files.length} files in ${dir}: ${files.join(", ")}`);
      files.forEach((f) => {
        const fullPath = path.join(dir, f);
        const types = getTypeFromFile(fullPath);
        types.forEach((typeInfo) => {
          codamaMap.set(typeInfo.name, typeInfo.members);
          l(`Added Codama type ${typeInfo.name} from ${fullPath}`);
        });
      });
    } else {
      console.warn(`Directory ${dir} does not exist`);
    }
  });

  l(`Codama types found: ${Array.from(codamaMap.keys()).join(", ")}`);

  let programs: string[] = [];
  config.src.forEach((srcItem) => {
    const baseDir = srcItem.directory;
    if (!fs.existsSync(baseDir)) {
      console.warn(`Source directory ${baseDir} does not exist`);
      return;
    }
    const excludes = srcItem.exclude;
    const subs = fs
      .readdirSync(baseDir, { withFileTypes: true })
      .filter((d) => d.isDirectory())
      .map((d) => d.name);
    const filtered = subs.filter(
      (p) =>
        !excludes.some((ex) =>
          ex.endsWith("*") ? p.startsWith(ex.slice(0, -1)) : p === ex,
        ),
    );
    programs = [...programs, ...filtered];
  });

  l(`Programs found: ${programs.join(", ")}`);

  programs.forEach((program) => {
    const interfaceDir = path.join(config.dist, program);
    const typesFile = path.join(interfaceDir, "types.ts");
    const accountsFile = path.join(interfaceDir, "accounts.ts");
    const instructionsFile = path.join(interfaceDir, "instructions.ts");

    if (
      !fs.existsSync(typesFile) ||
      !fs.existsSync(accountsFile) ||
      !fs.existsSync(instructionsFile)
    ) {
      console.warn(
        `Missing files for program ${program}: types=${fs.existsSync(typesFile)}, accounts=${fs.existsSync(accountsFile)}, instructions=${fs.existsSync(instructionsFile)}`,
      );
      return;
    }

    const typesSource = ts.createSourceFile(
      typesFile,
      fs.readFileSync(typesFile, "utf8"),
      ts.ScriptTarget.Latest,
      true,
    );
    const accountsSource = ts.createSourceFile(
      accountsFile,
      fs.readFileSync(accountsFile, "utf8"),
      ts.ScriptTarget.Latest,
      true,
    );
    const instructionsSource = ts.createSourceFile(
      instructionsFile,
      fs.readFileSync(instructionsFile, "utf8"),
      ts.ScriptTarget.Latest,
      true,
    );

    const typesInterfaces = getInterfaces(typesSource, typesFile);
    const accountsInterfaces = getInterfaces(accountsSource, accountsFile);
    const instructionsInterfaces = getInterfaces(
      instructionsSource,
      instructionsFile,
    );

    l(`Interfaces for ${program}:`);
    l(`  Types: ${Array.from(typesInterfaces.keys()).join(", ")}`);
    l(`  Accounts: ${Array.from(accountsInterfaces.keys()).join(", ")}`);
    l(
      `  Instructions: ${Array.from(instructionsInterfaces.keys()).join(", ")}`,
    );

    const pairs: Pair[] = [];

    function addPairs(
      interfaces: Map<string, ts.PropertySignature[]>,
      category: Pair["category"],
    ) {
      for (const [intName, intMembers] of interfaces) {
        const codamaName = intName.slice(1);
        const codamaMembers = codamaMap.get(codamaName);
        if (codamaMembers) {
          pairs.push({
            intName,
            codamaName,
            intMembers,
            codamaMembers,
            category,
          });
          l(`Matched ${intName} with Codama type ${codamaName}`);
        } else {
          console.warn(
            `Codama type ${codamaName} not found for ${intName} in ${category}`,
          );
        }
      }
    }

    addPairs(typesInterfaces, "types");
    addPairs(accountsInterfaces, "accounts");
    addPairs(instructionsInterfaces, "instructions");

    if (pairs.length === 0) {
      console.warn(
        `No type pairs found for program ${program}, skipping codec generation`,
      );
      return;
    }

    // Collect imports
    const typeIs: string[] = [];
    const accountIs: string[] = [];
    const instructionIs: string[] = [];
    const codamaTypesSet = new Set<string>();
    const primitiveTsSet = new Set<string>();

    pairs.forEach((pair) => {
      codamaTypesSet.add(pair.codamaName);
      if (pair.category === "types") typeIs.push(pair.intName);
      else if (pair.category === "accounts") accountIs.push(pair.intName);
      else instructionIs.push(pair.intName);
    });

    // Generate content
    const lines: string[] = [];

    if (typeIs.length > 0) {
      lines.push(`import { ${typeIs.join(", ")} } from "./types";`);
    }
    if (accountIs.length > 0) {
      lines.push(`import { ${accountIs.join(", ")} } from "./accounts";`);
    }
    if (instructionIs.length > 0) {
      lines.push(
        `import { ${instructionIs.join(", ")} } from "./instructions";`,
      );
    }
    if (codamaTypesSet.size > 0) {
      lines.push(
        `import { ${Array.from(codamaTypesSet).join(", ")} } from "../../codama";`,
      );
    }

    // Primitives will be inserted after generation
    lines.push("");

    pairs.forEach((pair) => {
      // enc
      const funcName = `enc${pair.codamaName}`;
      const arg =
        pair.category === "types"
          ? `x?: ${pair.intName}`
          : `x: ${pair.intName}`;
      const returnType = pair.codamaName;
      lines.push(`export function ${funcName}(${arg}): ${returnType} {`);
      const bodyLines: string[] = ["  return {"];

      const hasFlags = pair.codamaMembers.some(
        (m) => ts.isIdentifier(m.name) && m.name.text === "flags",
      );
      const flagFields: ts.PropertySignature[] = [];
      if (hasFlags) {
        pair.intMembers.forEach((m) => {
          if (!m.type || !ts.isIdentifier(m.name)) return;
          const iType = m.type.getText();
          const isBool = iType === "boolean";
          const isOpt = !!m.questionToken;
          if (isBool || (isOpt && !isBool)) {
            flagFields.push(m);
          }
        });
      }

      pair.codamaMembers.forEach((c) => {
        if (!c.type || !ts.isIdentifier(c.name)) {
          console.warn(
            `Skipping codama member in ${pair.codamaName}: missing type or non-identifier name`,
          );
          return;
        }
        const cName = c.name.text;
        let cType = c.type.getText();
        const baseType = cType.endsWith("Args") ? cType.slice(0, -4) : cType;

        bodyLines.push(`    ${cName}: `);
        if (cName === "flags") {
          primitiveTsSet.add("TBitFieldBuilder");
          let builderLine = bodyLines.pop() + "new TBitFieldBuilder()";
          bodyLines.push(builderLine);
          flagFields.forEach((f) => {
            if (!f.type || !ts.isIdentifier(f.name)) return;
            const fName = f.name.text;
            const isBool = f.type.getText() === "boolean";
            const isOpt = !!f.questionToken;
            let method = "";
            if (isBool && !isOpt) method = "withBool";
            else if (isBool && isOpt) method = "withOptBool";
            else method = "withOptNonBool";
            bodyLines.push(
              `      .${method}(${pair.category === "types" ? `x?.${fName}` : `x.${fName}`})`,
            );
          });
          bodyLines.push(`      .build().getRaw(),`);
        } else {
          const trans = primitiveMap[baseType];
          if (trans) {
            primitiveTsSet.add(trans);
            bodyLines[bodyLines.length - 1] +=
              `new ${trans}(${pair.category === "types" ? `x?.${cName}` : `x.${cName}`}).getRaw(),`;
          } else {
            bodyLines[bodyLines.length - 1] +=
              `enc${baseType}(${pair.category === "types" ? `x?.${cName}` : `x.${cName}`}),`;
          }
        }
      });

      bodyLines.push("  };");
      lines.push(...bodyLines);
      lines.push("}");
      lines.push("");

      if (pair.category !== "instructions") {
        // dec
        const decName = `dec${pair.codamaName}`;
        const decArg =
          pair.category === "types"
            ? `x?: ${pair.codamaName}`
            : `x: ${pair.codamaName}`;
        const decReturn = pair.intName;
        lines.push(`export function ${decName}(${decArg}): ${decReturn} {`);
        const decBody: string[] = ["  return {"];

        let flagIndex = 0;
        pair.intMembers.forEach((i) => {
          if (!i.type || !ts.isIdentifier(i.name)) {
            console.warn(
              `Skipping interface member in ${pair.intName}: missing type or non-identifier name`,
            );
            return;
          }
          const iName = i.name.text;
          const iType = i.type.getText();
          const isBool = iType === "boolean";
          const isOpt = !!i.questionToken;
          decBody.push(`    ${iName}: `);

          if (isBool || (isOpt && !isBool)) {
            primitiveTsSet.add("TBitField");
            decBody[decBody.length - 1] +=
              `new TBitField(${pair.category === "types" ? `x?.flags` : `x.flags`}).get(${flagIndex}),`;
            flagIndex++;
          } else {
            const cField = pair.codamaMembers.find(
              (cm) => ts.isIdentifier(cm.name) && cm.name.text === iName,
            );
            if (!cField || !cField.type) {
              console.warn(
                `Missing codama field or type for ${iName} in ${pair.codamaName}`,
              );
              return;
            }
            let cType = cField.type.getText();
            const baseType = cType.endsWith("Args")
              ? cType.slice(0, -4)
              : cType;
            const trans = primitiveMap[baseType];
            if (trans) {
              primitiveTsSet.add(trans);
              decBody[decBody.length - 1] +=
                `new ${trans}(${pair.category === "types" ? `x?.${iName}` : `x.${iName}`}).get(),`;
            } else {
              decBody[decBody.length - 1] +=
                `dec${baseType}(${pair.category === "types" ? `x?.${iName}` : `x.${iName}`}),`;
            }
          }
        });

        decBody.push("  };");
        lines.push(...decBody);
        lines.push("}");
        lines.push("");
      }
    });

    // Insert primitives import after codama
    if (primitiveTsSet.size > 0) {
      const primImport = `import { ${Array.from(primitiveTsSet).sort().join(", ")} } from "../../../interfaces/primitives";`;
      const insertIndex =
        lines.findIndex(
          (l) => l.startsWith("import { ") && l.includes('from "../../codama"'),
        ) + 1;
      lines.splice(insertIndex, 0, primImport);
    }

    const codecFile = path.join(interfaceDir, "codecs.ts");
    fs.writeFileSync(codecFile, lines.join("\n"));
    l(`Generated codecs for ${program} at ${codecFile}`);
  });
}

main();
