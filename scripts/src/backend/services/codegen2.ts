import * as fs from "fs";
import * as path from "path";
import ts from "typescript";
import { rootPath } from "../utils";

interface Config {
  src: Array<{ directory: string; exclude: string[] }>;
  dist: string;
}

type Category = "types" | "accounts" | "instructions";

interface TypeInfo {
  name: string;
  interfaceName: string;
  codamaName: string;
  fields: FieldInfo[];
  category: Category;
}

interface FieldInfo {
  name: string;
  type: string;
  isOptional: boolean;
  isArray: boolean;
  isBoolean: boolean;
}

interface CodamaFieldInfo {
  name: string;
  type: string;
  isArray: boolean;
}

function loadConfig(configPath: string): Config {
  const content = fs.readFileSync(configPath, "utf-8");
  return JSON.parse(content);
}

function findInterfaceTypes(distPath: string): Map<string, TypeInfo> {
  const typesMap = new Map<string, TypeInfo>();
  const categories: Array<Category> = ["types", "accounts", "instructions"];

  for (const category of categories) {
    const filePath = path.join(distPath, `${category}.ts`);
    if (!fs.existsSync(filePath)) continue;

    const sourceFile = ts.createSourceFile(
      filePath,
      fs.readFileSync(filePath, "utf-8"),
      ts.ScriptTarget.Latest,
      true,
    );

    ts.forEachChild(sourceFile, (node) => {
      if (ts.isInterfaceDeclaration(node) && node.name.text.startsWith("I")) {
        const interfaceName = node.name.text;
        const codamaName = interfaceName.substring(1);
        const fields = extractFields(node);

        typesMap.set(interfaceName, {
          name: interfaceName,
          interfaceName,
          codamaName,
          fields,
          category,
        });
      }
    });
  }

  return typesMap;
}

function extractFields(node: ts.InterfaceDeclaration): FieldInfo[] {
  const fields: FieldInfo[] = [];

  node.members.forEach((member) => {
    if (
      ts.isPropertySignature(member) &&
      member.name &&
      ts.isIdentifier(member.name)
    ) {
      const name = member.name.text;
      const isOptional = !!member.questionToken;
      let type = "unknown";
      let isArray = false;
      let isBoolean = false;

      if (member.type) {
        type = member.type.getText();
        isArray = ts.isArrayTypeNode(member.type);
        isBoolean = member.type.kind === ts.SyntaxKind.BooleanKeyword;
      }

      fields.push({ name, type, isOptional, isArray, isBoolean });
    }
  });

  return fields;
}

function findCodamaTypes(distPath: string): Map<string, CodamaFieldInfo[]> {
  const codamaPath = distPath.replace(/codegen$/, "codama");
  const codamaTypes = new Map<string, CodamaFieldInfo[]>();
  const categories: Category[] = ["types", "accounts", "instructions"];

  for (const category of categories) {
    const categoryPath = path.join(codamaPath, category);
    if (!fs.existsSync(categoryPath)) continue;

    const files = fs.readdirSync(categoryPath).filter((f) => f.endsWith(".ts"));

    for (const file of files) {
      const filePath = path.join(categoryPath, file);
      const sourceFile = ts.createSourceFile(
        filePath,
        fs.readFileSync(filePath, "utf-8"),
        ts.ScriptTarget.Latest,
        true,
      );

      ts.forEachChild(sourceFile, (node) => {
        if (ts.isTypeAliasDeclaration(node)) {
          const typeName = node.name.text;
          const fields = extractCodamaFields(node.type);
          codamaTypes.set(typeName, fields);
        }
      });
    }
  }

  return codamaTypes;
}

function extractCodamaFields(typeNode: ts.TypeNode): CodamaFieldInfo[] {
  const fields: CodamaFieldInfo[] = [];

  if (ts.isTypeLiteralNode(typeNode)) {
    typeNode.members.forEach((member) => {
      if (
        ts.isPropertySignature(member) &&
        member.name &&
        ts.isIdentifier(member.name)
      ) {
        const name = member.name.text;
        let type = "unknown";
        let isArray = false;

        if (member.type) {
          type = member.type.getText();
          isArray = ts.isArrayTypeNode(member.type);
        }

        fields.push({ name, type, isArray });
      }
    });
  }

  return fields;
}

function getTransformerClass(type: string): string | null {
  if (type.includes("bigint") || type === "Uint64") return "TUint64";
  if (type.includes("Uint128")) return "TUint128";
  if (type === "number" || type === "Uint32") return "TUint32";
  if (type === "Uint16") return "TUint16";
  if (type === "Uint8") return "TUint8";
  if (type.includes("Address")) return "TAddress";
  if (type.includes("String16")) return "TString16";
  if (type.includes("String32")) return "TString32";
  if (type.includes("String64")) return "TString64";
  if (type.includes("String4096")) return "TString4096";
  return null;
}

function hasFlagsField(codamaFields: CodamaFieldInfo[]): boolean {
  return codamaFields.some((f) => f.name === "flags" && f.type === "BitField");
}

function generateEncoder(
  typeInfo: TypeInfo,
  codamaFields: CodamaFieldInfo[],
  allTypes: Map<string, TypeInfo>,
): string {
  const isOptional = typeInfo.category === "types";
  const paramPrefix = isOptional ? "x?:" : "x:";

  let body = "  return {\n";
  const hasFlags = hasFlagsField(codamaFields);

  if (hasFlags) {
    const flagFields = typeInfo.fields.filter(
      (f) =>
        f.isBoolean ||
        (f.isOptional && !f.isBoolean) ||
        (f.isOptional && f.isBoolean),
    );

    if (flagFields.length > 0) {
      body += "    flags: new TBitFieldBuilder()\n";
      flagFields.forEach((field) => {
        if (field.isBoolean && !field.isOptional) {
          body += `      .withBool(x.${field.name})\n`;
        } else if (field.isOptional && field.isBoolean) {
          body += `      .withOptBool(x.${field.name})\n`;
        } else if (field.isOptional) {
          body += `      .withOptNonBool(x.${field.name})\n`;
        }
      });
      body += "      .build()\n";
      body += "      .getRaw(),\n";
    }
  }

  for (const codamaField of codamaFields) {
    if (codamaField.name === "flags") continue;

    const interfaceField = typeInfo.fields.find(
      (f) => f.name === codamaField.name || codamaField.name.startsWith(f.name),
    );

    if (!interfaceField) continue;

    const transformer = getTransformerClass(interfaceField.type);
    const prefix = isOptional ? "x?." : "x.";

    if (transformer) {
      body += `    ${codamaField.name}: new ${transformer}(${prefix}${interfaceField.name}).getRaw(),\n`;
    } else {
      // Check if it's a custom type
      const customType = Array.from(allTypes.values()).find((t) =>
        interfaceField.type.includes(t.interfaceName),
      );
      if (customType) {
        const encoderName = `enc${customType.codamaName}`;
        body += `    ${codamaField.name}: ${encoderName}(${prefix}${interfaceField.name}),\n`;
      }
    }
  }

  body += "  };";

  return `export function enc${typeInfo.codamaName}(${paramPrefix} ${typeInfo.interfaceName}): ${typeInfo.codamaName} {\n${body}\n}`;
}

function generateDecoder(
  typeInfo: TypeInfo,
  codamaFields: CodamaFieldInfo[],
  allTypes: Map<string, TypeInfo>,
): string {
  const isOptional = typeInfo.category === "types";
  const paramPrefix = isOptional ? "x?:" : "x:";

  let body = "  return {\n";
  const hasFlags = hasFlagsField(codamaFields);

  let flagIndex = 0;
  for (const field of typeInfo.fields) {
    if (
      hasFlags &&
      (field.isBoolean ||
        (field.isOptional && !field.isBoolean) ||
        (field.isOptional && field.isBoolean))
    ) {
      const prefix = isOptional ? "x?." : "x.";
      body += `    ${field.name}: new TBitField(${prefix}flags).get(${flagIndex}),\n`;
      flagIndex++;
      continue;
    }

    const transformer = getTransformerClass(field.type);
    const prefix = isOptional ? "x?." : "x.";

    if (transformer) {
      body += `    ${field.name}: new ${transformer}(${prefix}${field.name}).get(),\n`;
    } else {
      const customType = Array.from(allTypes.values()).find((t) =>
        field.type.includes(t.interfaceName),
      );
      if (customType) {
        const decoderName = `dec${customType.codamaName}`;
        body += `    ${field.name}: ${decoderName}(${prefix}${field.name}),\n`;
      }
    }
  }

  body += "  };";

  return `export function dec${typeInfo.codamaName}(${paramPrefix} ${typeInfo.codamaName}): ${typeInfo.interfaceName} {\n${body}\n}`;
}

function generateCodecs(
  typesMap: Map<string, TypeInfo>,
  codamaTypes: Map<string, CodamaFieldInfo[]>,
): string {
  const imports = new Set<string>();
  const transformers = new Set<string>();
  const interfaceImports: Map<string, Set<string>> = new Map([
    ["types", new Set()],
    ["accounts", new Set()],
    ["instructions", new Set()],
  ]);
  const codamaImports = new Set<string>();

  // Collect all necessary imports
  for (const [_, typeInfo] of typesMap) {
    interfaceImports.get(typeInfo.category)!.add(typeInfo.interfaceName);
    codamaImports.add(typeInfo.codamaName);

    for (const field of typeInfo.fields) {
      const transformer = getTransformerClass(field.type);
      if (transformer) {
        transformers.add(transformer);
      }
    }

    const codamaFields = codamaTypes.get(typeInfo.codamaName);
    if (codamaFields && hasFlagsField(codamaFields)) {
      transformers.add("TBitField");
      transformers.add("TBitFieldBuilder");
    }
  }

  // Generate import statements
  let output = "";

  for (const [category, names] of interfaceImports) {
    if (names.size > 0) {
      output += `import { ${Array.from(names).join(", ")} } from "./${category}";\n`;
    }
  }

  if (codamaImports.size > 0) {
    output += `import {\n  ${Array.from(codamaImports).join(",\n  ")}\n} from "../../codama";\n`;
  }

  if (transformers.size > 0) {
    output += `import {\n  ${Array.from(transformers).sort().join(",\n  ")}\n} from "../../../interfaces/primitives";\n`;
  }

  output += "\n";

  // Generate codecs in order: types, accounts, instructions
  const categories: Array<Category> = ["types", "accounts", "instructions"];

  for (const category of categories) {
    for (const [_, typeInfo] of typesMap) {
      if (typeInfo.category !== category) continue;

      const codamaFields = codamaTypes.get(typeInfo.codamaName);
      if (!codamaFields) continue;

      output += generateEncoder(typeInfo, codamaFields, typesMap) + "\n\n";

      if (category !== "instructions") {
        output += generateDecoder(typeInfo, codamaFields, typesMap) + "\n\n";
      }
    }
  }

  return output.trim() + "\n";
}

function main() {
  const configPath = rootPath("./src/backend/services/path.json");
  const config = loadConfig(configPath);

  // Process each program directory
  for (const src of config.src) {
    const packagesDir = path.resolve(rootPath(src.directory));
    if (!fs.existsSync(packagesDir)) continue;

    const programs = fs.readdirSync(packagesDir).filter((name) => {
      const isExcluded = src.exclude.some((pattern) => {
        if (pattern.endsWith("*")) {
          return name.startsWith(pattern.slice(0, -1));
        }
        return name === pattern;
      });
      return (
        !isExcluded && fs.statSync(path.join(packagesDir, name)).isDirectory()
      );
    });

    for (const program of programs) {
      const programDistPath = path.join(config.dist, program);
      if (!fs.existsSync(programDistPath)) continue;

      console.log(`Processing program: ${program}`);

      const typesMap = findInterfaceTypes(programDistPath);
      const codamaTypes = findCodamaTypes(config.dist);

      const codecContent = generateCodecs(typesMap, codamaTypes);

      const codecPath = path.join(programDistPath, "codecs.ts");
      fs.writeFileSync(codecPath, codecContent);

      console.log(`Generated codecs: ${codecPath}`);
    }
  }
}

main();
