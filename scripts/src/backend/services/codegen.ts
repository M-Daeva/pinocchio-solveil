import * as fs from "fs/promises";
import * as path from "path";
import { rootPath } from "../utils";
import { l } from "../../common/utils";

interface PathConfig {
  src: Array<{
    directory: string;
    exclude: string[];
  }>;
  dist: string;
}

type ParsedStructType = "CodamaType" | "CodamaAccount" | "CodamaInstruction";

interface ParsedStruct {
  type: ParsedStructType;
  name: string;
  codamaName?: string | undefined;
  fields: Field[];
  optionalFields?: string[] | undefined;
  bitFieldFields?: string[] | undefined;
  enumFields?: string[] | undefined;
}

interface ParsedEnum {
  type: "CodamaInstructions";
  name: string;
  variants: EnumVariant[];
}

interface EnumVariant {
  name: string;
  fields: Field[];
}

interface Field {
  name: string;
  rustType: string;
  isOptional?: boolean;
  optionalList?: string[];
}

// Read and parse config
async function loadConfig(configPath: string): Promise<PathConfig> {
  const content = await fs.readFile(configPath, "utf-8");
  return JSON.parse(content);
}

// Convert snake_case to camelCase
function toCamelCase(str: string): string {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

// Convert Rust type to TypeScript type
function convertType(
  rustType: string,
  allStructNames: Set<string> = new Set(),
): string {
  // Remove whitespace
  rustType = rustType.trim();

  // Handle arrays [Type; length]
  const arrayMatch = rustType.match(/^\[(.+?);\s*\d+\]$/);
  if (arrayMatch && arrayMatch[1]) {
    const innerType = convertType(arrayMatch[1], allStructNames);
    // Add 'I' prefix if it's a custom type (starts with uppercase) and doesn't already have it
    const convertedInner =
      /^[A-Z]/.test(innerType) && !innerType.startsWith("I")
        ? `I${innerType}`
        : innerType;
    return `${convertedInner}[]`;
  }

  // Basic type mappings
  const typeMap: Record<string, string> = {
    u8: "number",
    Uint16: "number",
    Uint32: "number",
    Uint64: "bigint",
    Uint128: "bigint",
    Pubkey: "Address",
    String16: "string",
    String32: "string",
    String64: "string",
    String4096: "string",
    BitField: "boolean",
  };

  // If it's a basic type, return the mapping
  if (typeMap[rustType]) {
    return typeMap[rustType] as string;
  }

  // If it's a custom type (starts with uppercase), add 'I' prefix only if it doesn't have it
  if (/^[A-Z]/.test(rustType) && !rustType.startsWith("I")) {
    return `I${rustType}`;
  }

  return rustType;
}

// Check if crate should be excluded
function shouldExcludeCrate(
  crateName: string,
  excludePatterns: string[],
): boolean {
  return excludePatterns.some((pattern) => {
    if (pattern.endsWith("*")) {
      return crateName.startsWith(pattern.slice(0, -1));
    }
    return crateName === pattern;
  });
}

// Remove comments from Rust code
function removeComments(content: string): string {
  // Remove single-line comments
  content = content.replace(/\/\/.*$/gm, "");
  // Remove multi-line comments
  content = content.replace(/\/\*[\s\S]*?\*\//g, "");
  return content;
}

// Extract derives from a struct/enum declaration
function extractDerives(declaration: string): string[] {
  const derives: string[] = [];
  const deriveRegex = /#\[derive\((.*?)\)\]/g;
  let match;

  while ((match = deriveRegex.exec(declaration)) !== null) {
    if (match[1]) {
      const deriveList = match[1].split(",").map((d) => d.trim());
      derives.push(...deriveList);
    }
  }

  return derives;
}

// Extract codama.name attribute
function extractCodamaName(declaration: string): string | undefined {
  const match = declaration.match(/#\[codama\(name\s*=\s*"([^"]+)"\)\]/);
  return match ? match[1] : undefined;
}

// Extract optional fields from OptionFlag
function extractOptionalFields(declaration: string): string[] {
  const match = declaration.match(/#\[optional\((.*?)\)\]/);
  if (!match || !match[1]) return [];
  return match[1].split(",").map((f) => f.trim());
}

// Extract enum fields from enumfields attribute
function extractEnumFields(declaration: string): string[] {
  const match = declaration.match(/#\[enumfields\((.*?)\)\]/);
  if (!match || !match[1]) return [];
  return match[1].split(",").map((f) => f.trim());
}

// Parse struct fields
function parseStructFields(structBody: string): Field[] {
  const fields: Field[] = [];
  const fieldRegex = /((?:#\[.*?\]\s*)*)pub\s+(\w+):\s*([^,\n]+),?/gs;
  let match;

  while ((match = fieldRegex.exec(structBody)) !== null) {
    const [, fieldAttributes, fieldName, fieldType] = match;

    if (!fieldName || !fieldType) continue;

    const trimmedFieldType = fieldType.trim().replace(/,$/, "");

    const optionalList = extractOptionalFields(fieldAttributes || "");

    const field: Field = {
      name: fieldName,
      rustType: trimmedFieldType,
    };

    if (optionalList.length > 0) {
      field.optionalList = optionalList;
    }

    fields.push(field);
  }

  return fields;
}

// Parse structs from Rust file
function parseStructs(content: string): ParsedStruct[] {
  const structs: ParsedStruct[] = [];
  content = removeComments(content);

  // Match struct declarations with their full content including attributes
  const structRegex =
    /((?:#\[.*?\]\s*)*)\s*pub\s+struct\s+(\w+)(?:\((.*?)\))?\s*\{([^}]*)\}/gs;
  let match;

  while ((match = structRegex.exec(content)) !== null) {
    const [, attributes, name, tupleContent, body] = match;

    if (!attributes || !name) continue;

    const derives = extractDerives(attributes);

    let structType: ParsedStructType | null = null;

    if (derives.includes("CodamaType")) structType = "CodamaType";
    else if (derives.includes("CodamaAccount")) structType = "CodamaAccount";
    else if (derives.includes("CodamaInstruction"))
      structType = "CodamaInstruction";

    if (!structType) continue;

    const structOptionalFields = extractOptionalFields(attributes);
    const enumFields = extractEnumFields(attributes);
    const codamaName = extractCodamaName(attributes);

    // Handle EnumWrapper (tuple struct)
    if (
      derives.includes("EnumWrapper") &&
      tupleContent &&
      enumFields.length > 0
    ) {
      structs.push({
        type: structType,
        name,
        fields: [],
        enumFields,
      });
      continue;
    }

    const fields = body ? parseStructFields(body) : [];

    let bitFieldFields: string[] = [];

    if (derives.includes("OptionFlag")) {
      const bitFields = fields.filter(
        (f) => f.rustType === "BitField" && f.optionalList,
      );
      for (const bitField of bitFields) {
        const optionalList = bitField.optionalList || [];
        for (const f of fields) {
          if (optionalList.includes(f.name)) {
            f.isOptional = true;
          }
        }
        const newFlags = optionalList.filter(
          (name) => !fields.find((f) => f.name === name),
        );
        bitFieldFields.push(...newFlags);
        // Remove the bitField field
        const index = fields.indexOf(bitField);
        if (index !== -1) {
          fields.splice(index, 1);
        }
      }
    }

    // Handle struct-level optional fields
    for (const f of fields) {
      if (structOptionalFields.includes(f.name)) {
        f.isOptional = true;
      }
    }

    structs.push({
      type: structType,
      name,
      codamaName,
      fields,
      optionalFields: structOptionalFields,
      bitFieldFields,
    });
  }

  return structs;
}

// Parse enums (CodamaInstructions)
function parseEnums(content: string): ParsedEnum[] {
  const enums: ParsedEnum[] = [];
  content = removeComments(content);

  const enumRegex = /((?:#\[.*?\]\s*)*)\s*pub\s+enum\s+(\w+)\s*\{([^}]+)\}/gs;
  let match;

  while ((match = enumRegex.exec(content)) !== null) {
    const [, attributes, name, body] = match;

    if (!attributes || !name || !body) continue;

    const derives = extractDerives(attributes);

    if (!derives.includes("CodamaInstructions")) continue;

    const variants: EnumVariant[] = [];
    const variantRegex = /(\w+)\s*\{([^}]*)\}/g;
    let variantMatch;

    while ((variantMatch = variantRegex.exec(body)) !== null) {
      const [, variantName, variantBody] = variantMatch;

      if (!variantName || !variantBody) continue;

      const fields = parseStructFields(variantBody);
      variants.push({ name: variantName, fields });
    }

    enums.push({
      type: "CodamaInstructions",
      name,
      variants,
    });
  }

  return enums;
}

// Collect used types from fields
function collectUsedTypes(fields: Field[]): Set<string> {
  const usedTypes = new Set<string>();

  for (const field of fields) {
    let rustType = field.rustType.trim();

    // Extract type from arrays [Type; length]
    const arrayMatch = rustType.match(/^\[(.+?);\s*\d+\]$/);
    if (arrayMatch && arrayMatch[1]) {
      rustType = arrayMatch[1].trim();
    }

    // If it's a custom type (starts with uppercase and not a basic type)
    if (
      /^[A-Z]/.test(rustType) &&
      ![
        "Pubkey",
        "Uint16",
        "Uint32",
        "Uint64",
        "Uint128",
        "String16",
        "String32",
        "String64",
        "String4096",
        "BitField",
      ].includes(rustType)
    ) {
      usedTypes.add(`I${rustType}`);
    }
  }

  return usedTypes;
}

// Generate TypeScript interface
function generateInterface(
  parsed:
    | ParsedStruct
    | { name: string; fields: Field[]; bitFieldFields?: string[] },
  interfaceName: string,
): string {
  let output = `export interface ${interfaceName} {\n`;

  // Add BitField boolean flags first
  if (parsed.bitFieldFields && parsed.bitFieldFields.length > 0) {
    for (const field of parsed.bitFieldFields) {
      const camelCaseField = toCamelCase(field);
      output += `  ${camelCaseField}: boolean;\n`;
    }
  }

  // Add regular fields
  for (const field of parsed.fields) {
    const tsType = convertType(field.rustType);
    const optional = field.isOptional ? "?" : "";
    const camelCaseField = toCamelCase(field.name);
    output += `  ${camelCaseField}${optional}: ${tsType};\n`;
  }

  output += "}\n";
  return output;
}

// Generate TypeScript enum
function generateEnum(parsed: ParsedStruct, enumName: string): string {
  if (!parsed.enumFields) return "";

  let output = `export enum ${enumName} {\n`;
  for (const field of parsed.enumFields) {
    // Capitalize first letter
    const capitalizedField = field.charAt(0).toUpperCase() + field.slice(1);
    output += `  ${capitalizedField},\n`;
  }
  output += "}\n";
  return output;
}

// Process a single Rust file
async function processRustFile(
  filePath: string,
): Promise<{ types: string[]; accounts: string[]; instructions: string[] }> {
  const content = await fs.readFile(filePath, "utf-8");
  const structs = parseStructs(content);
  const enums = parseEnums(content);

  const types: string[] = [];
  const accounts: string[] = [];
  const instructions: string[] = [];

  // Process structs
  for (const struct of structs) {
    if (struct.enumFields && struct.enumFields.length > 0) {
      // EnumWrapper
      const enumName = `I${struct.name}`;
      const enumCode = generateEnum(struct, enumName);
      types.push(enumCode);
    } else if (struct.type === "CodamaType") {
      const interfaceName = `I${struct.name}`;
      types.push(generateInterface(struct, interfaceName));
    } else if (struct.type === "CodamaAccount") {
      const interfaceName = `I${struct.name}`;
      accounts.push(generateInterface(struct, interfaceName));
    } else if (struct.type === "CodamaInstruction") {
      const codamaName = struct.codamaName || struct.name;
      const interfaceName = `I${codamaName}InstructionDataArgs`;
      instructions.push(generateInterface(struct, interfaceName));
    }
  }

  // Process enums (CodamaInstructions)
  for (const enumData of enums) {
    for (const variant of enumData.variants) {
      const interfaceName = `I${variant.name}InstructionDataArgs`;
      instructions.push(generateInterface(variant, interfaceName));
    }
  }

  return { types, accounts, instructions };
}

// Process a crate directory
async function processCrate(crateDir: string, distDir: string): Promise<void> {
  const crateName = path.basename(crateDir);
  const outputDir = path.join(distDir, crateName);

  let allTypes: string[] = [];
  let allAccounts: string[] = [];
  let allInstructions: string[] = [];
  let allStructs: ParsedStruct[] = [];

  // Find all .rs files recursively
  async function findRustFiles(dir: string): Promise<string[]> {
    const files: string[] = [];
    const entries = await fs.readdir(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        files.push(...(await findRustFiles(fullPath)));
      } else if (entry.isFile() && entry.name.endsWith(".rs")) {
        files.push(fullPath);
      }
    }

    return files;
  }

  const rustFiles = await findRustFiles(crateDir);

  for (const file of rustFiles) {
    const content = await fs.readFile(file, "utf-8");
    const structs = parseStructs(content);
    allStructs.push(...structs);

    const { types, accounts, instructions } = await processRustFile(file);
    allTypes.push(...types);
    allAccounts.push(...accounts);
    allInstructions.push(...instructions);
  }

  // Write output files if there's content
  if (
    allTypes.length > 0 ||
    allAccounts.length > 0 ||
    allInstructions.length > 0
  ) {
    await fs.mkdir(outputDir, { recursive: true });

    if (allTypes.length > 0) {
      const content =
        'import { Address } from "gill";\n\n' + allTypes.join("\n");
      await fs.writeFile(path.join(outputDir, "types.ts"), content);
    }

    if (allAccounts.length > 0) {
      // Collect used types in accounts
      const usedTypes = new Set<string>();
      for (const struct of allStructs) {
        if (struct.type === "CodamaAccount") {
          const types = collectUsedTypes(struct.fields);
          types.forEach((t) => usedTypes.add(t));
        }
      }

      let imports = 'import { Address } from "gill";\n';
      if (usedTypes.size > 0) {
        imports += `import { ${Array.from(usedTypes).join(", ")} } from "./types";\n`;
      }

      const content = imports + "\n" + allAccounts.join("\n");
      await fs.writeFile(path.join(outputDir, "accounts.ts"), content);
    }

    if (allInstructions.length > 0) {
      // Collect used types in instructions
      const usedTypes = new Set<string>();
      for (const struct of allStructs) {
        if (struct.type === "CodamaInstruction") {
          const types = collectUsedTypes(struct.fields);
          types.forEach((t) => usedTypes.add(t));
        }
      }

      let imports = 'import { Address } from "gill";\n';
      if (usedTypes.size > 0) {
        imports += `import { ${Array.from(usedTypes).join(", ")} } from "./types";\n`;
      }

      const content = imports + "\n" + allInstructions.join("\n");
      await fs.writeFile(path.join(outputDir, "instructions.ts"), content);
    }
  }
}

async function main() {
  const start = Date.now();

  const config = await loadConfig(rootPath("./src/backend/services/path.json"));

  for (const srcConfig of config.src) {
    const srcDir = rootPath(srcConfig.directory);

    const isSrcExists = await fs.exists(srcDir);
    if (!isSrcExists) {
      console.warn(`Source directory not found: ${srcDir}`);
      continue;
    }

    const crates = (await fs.readdir(srcDir, { withFileTypes: true }))
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name);

    for (const crate of crates) {
      if (shouldExcludeCrate(crate, srcConfig.exclude)) {
        l(`Skipping excluded crate: ${crate}`);
        continue;
      }

      const crateDir = path.join(srcDir, crate);
      l(`Processing crate: ${crate}`);
      await processCrate(crateDir, rootPath(config.dist));
    }
  }

  l("Code generation complete!");

  l({ time: (Date.now() - start) / 1e3 });
}

main();
