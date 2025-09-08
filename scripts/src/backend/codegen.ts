import ts from "typescript";
import * as path from "path";

interface TypeInfo {
  name: string;
  properties: PropertyInfo[];
  isOptional?: boolean;
}

interface PropertyInfo {
  name: string;
  type: string;
  isOptional: boolean;
  isArray: boolean;
  isEnum: boolean;
  enumValues?: string[];
}

interface FunctionInfo {
  name: string;
  parameters: ParameterInfo[];
}

interface ParameterInfo {
  name: string;
  type: string;
  isOptional: boolean;
}

export class AnchorTypeGenerator {
  private sourceFile: ts.SourceFile;
  private typeChecker: ts.TypeChecker;
  private program: ts.Program;

  constructor(filePath: string) {
    this.program = ts.createProgram([filePath], {
      target: ts.ScriptTarget.ES2018,
      module: ts.ModuleKind.CommonJS,
    });
    this.typeChecker = this.program.getTypeChecker();
    this.sourceFile = this.program.getSourceFile(filePath)!;
  }

  getSrcImports(types: TypeInfo[]): string {
    const importPath =
      "./" + path.basename(this.sourceFile.fileName).replace(/\.ts$/, "");

    let typeImports = types
      .reduce((acc, cur) => acc + cur.name + ", ", "")
      .trim();

    typeImports = typeImports.slice(0, typeImports.length - 1);

    return `import { ${typeImports} } from '${importPath}';\n\n`;
  }

  generate(): string {
    const types = this.extractTypes();
    const functions = this.extractFunctions();

    let output = `// Auto-generated Anchor types and converters\n`;
    output += `import * as anchor from '@coral-xyz/anchor';\n`;
    output += `import { PublicKey } from '@solana/web3.js';\n`;
    output += this.getSrcImports(types);

    // Generate anchor types
    output += this.generateAnchorTypes(types);

    // Generate converter functions
    output += this.generateConverters(types, functions);

    return output;
  }

  private extractTypes(): TypeInfo[] {
    const types: TypeInfo[] = [];

    const visit = (node: ts.Node) => {
      if (ts.isInterfaceDeclaration(node) && node.name) {
        const typeInfo = this.extractTypeInfo(node);
        if (typeInfo) {
          types.push(typeInfo);
        }
      }
      ts.forEachChild(node, visit);
    };

    visit(this.sourceFile);
    return types;
  }

  private extractFunctions(): FunctionInfo[] {
    const functions: FunctionInfo[] = [];

    const visit = (node: ts.Node) => {
      if (ts.isFunctionDeclaration(node) && node.name) {
        const functionInfo = this.extractFunctionInfo(node);
        if (functionInfo) {
          functions.push(functionInfo);
        }
      }
      ts.forEachChild(node, visit);
    };

    visit(this.sourceFile);
    return functions;
  }

  private extractTypeInfo(node: ts.InterfaceDeclaration): TypeInfo | null {
    const name = node.name.text;
    const properties: PropertyInfo[] = [];

    for (const member of node.members) {
      if (ts.isPropertySignature(member) && member.name) {
        const propInfo = this.extractPropertyInfo(member);
        if (propInfo) {
          properties.push(propInfo);
        }
      }
    }

    return { name, properties };
  }

  private extractPropertyInfo(prop: ts.PropertySignature): PropertyInfo | null {
    if (!prop.name || !ts.isIdentifier(prop.name)) return null;

    const name = prop.name.text;
    const isOptional = !!prop.questionToken;
    const type = this.typeChecker.getTypeAtLocation(prop);
    let typeString = this.typeChecker.typeToString(type);

    // Check if the original source text contains N<T> pattern
    // since TypeScript resolves N<64> to just "number"
    if (prop.type && typeString === "number") {
      const sourceText = prop.type.getText();
      if (sourceText.match(/N<\d+>/)) {
        typeString = sourceText;
      }
    }

    return {
      name,
      type: typeString,
      isOptional,
      isArray: this.isArrayType(type),
      isEnum: this.isEnumType(type),
      enumValues: this.getEnumValues(type),
    };
  }

  private extractFunctionInfo(
    node: ts.FunctionDeclaration
  ): FunctionInfo | null {
    if (!node.name) return null;

    const name = node.name.text;
    const parameters: ParameterInfo[] = [];

    for (const param of node.parameters) {
      if (ts.isIdentifier(param.name)) {
        const paramName = param.name.text;
        const isOptional = !!param.questionToken;
        const type = this.typeChecker.getTypeAtLocation(param);
        const typeString = this.typeChecker.typeToString(type);

        parameters.push({
          name: paramName,
          type: typeString,
          isOptional,
        });
      }
    }

    return { name, parameters };
  }

  private isArrayType(type: ts.Type): boolean {
    return this.typeChecker.isArrayType(type);
  }

  private isEnumType(type: ts.Type): boolean {
    return !!(type.flags & ts.TypeFlags.EnumLike);
  }

  private getEnumValues(type: ts.Type): string[] | undefined {
    if (this.isEnumType(type)) {
      return type.symbol?.valueDeclaration &&
        ts.isEnumDeclaration(type.symbol.valueDeclaration)
        ? type.symbol.valueDeclaration.members.map((m) =>
            ts.isIdentifier(m.name) ? m.name.text : ""
          )
        : undefined;
    }
    return undefined;
  }

  private generateAnchorTypes(types: TypeInfo[]): string {
    let output = `// Anchor-generated types\n`;

    for (const type of types) {
      output += this.generateAnchorType(type);
    }

    return output + "\n";
  }

  private generateAnchorType(type: TypeInfo): string {
    const anchorTypeName = `Anchor${type.name}`;

    // Check if this should be a tuple type (for function arguments)
    if (type.name.endsWith("Args")) {
      return this.generateTupleType(type, anchorTypeName);
    }

    // Generate interface type
    let output = `export interface ${anchorTypeName} {\n`;

    for (const prop of type.properties) {
      const anchorType = this.convertToAnchorType(prop);
      const optional = prop.isOptional ? "?" : "";
      output += `  ${prop.name}${optional}: ${anchorType};\n`;
    }

    output += `}\n\n`;
    return output;
  }

  private generateTupleType(type: TypeInfo, anchorTypeName: string): string {
    const tupleTypes = type.properties.map((prop) => {
      const anchorType = this.convertToAnchorType(prop);
      return prop.isOptional ? `${anchorType} | null` : anchorType;
    });

    return `export type ${anchorTypeName} = [\n  ${tupleTypes.join(
      ",\n  "
    )}\n];\n\n`;
  }

  private convertToAnchorType(prop: PropertyInfo): string {
    const baseType = this.getBaseType(prop.type);

    // Handle arrays first, before other type conversions
    if (prop.isArray) {
      const elementType = this.extractArrayElementType(prop.type);
      const elementProp: PropertyInfo = {
        ...prop,
        type: elementType,
        isArray: false,
      };
      const elementAnchorType = this.convertToAnchorType(elementProp);
      return `${elementAnchorType}[]`;
    }

    // Handle N<T> sized number types
    const nTypeMatch = this.extractNType(baseType);
    if (nTypeMatch) {
      const size = parseInt(nTypeMatch);
      // N<64> should be anchor.BN, smaller sizes remain number
      return size >= 64 ? "anchor.BN" : "number";
    }

    // Handle specific type conversions
    switch (baseType) {
      case "number":
        // Numbers that represent amounts/balances should be BN
        if (
          prop.name.includes("amount") ||
          prop.name.includes("balance") ||
          prop.name.includes("fee") ||
          prop.name.includes("value")
        ) {
          return "anchor.BN";
        }
        return "number";

      case "bigint":
        return "anchor.BN";

      case "string":
        // Strings that represent addresses should be PublicKey
        if (
          prop.name.includes("address") ||
          prop.name.includes("key") ||
          prop.name.includes("mint") ||
          prop.name.includes("account")
        ) {
          return "PublicKey";
        }
        return "string";

      case "PublicKey":
        return "PublicKey";

      case "boolean":
        return "boolean";

      default:
        // Handle custom types
        if (this.isCustomType(baseType)) {
          return `Anchor${baseType}`;
        }

        // Handle enums
        if (prop.isEnum && prop.enumValues) {
          return `{ ${prop.enumValues
            .map((v) => `${v.toLowerCase()}: {}`)
            .join(" | ")} }`;
        }

        return baseType;
    }
  }

  /**
   * Extract the size parameter from N<T> type
   * @param typeString - The type string to check
   * @returns The size as string if it's an N<T> type, null otherwise
   */
  private extractNType(typeString: string): string | null {
    const nTypeMatch = typeString.match(/^N<(\d+)>$/);
    return nTypeMatch ? nTypeMatch[1] : null;
  }

  /**
   * Check if a type is a sized number type N<T>
   * @param typeString - The type string to check
   * @returns true if it's an N<T> type
   */
  private isNType(typeString: string): boolean {
    return /^N<\d+>$/.test(typeString);
  }

  private getBaseType(typeString: string): string {
    // Remove optional markers and whitespace
    let type = typeString.replace(/\s*\|\s*undefined/g, "").trim();

    // Handle array types
    if (type.endsWith("[]")) {
      return type.slice(0, -2);
    }

    // Handle Array<T> syntax
    const arrayMatch = type.match(/^Array<(.+)>$/);
    if (arrayMatch) {
      return arrayMatch[1];
    }

    return type;
  }

  private extractArrayElementType(typeString: string): string {
    if (typeString.endsWith("[]")) {
      return typeString.slice(0, -2);
    }

    const arrayMatch = typeString.match(/^Array<(.+)>$/);
    if (arrayMatch) {
      return arrayMatch[1];
    }

    return typeString;
  }

  private isCustomType(type: string): boolean {
    const primitiveTypes = [
      "number",
      "string",
      "boolean",
      "bigint",
      "PublicKey",
    ];
    return (
      !primitiveTypes.includes(type) &&
      !type.includes("|") &&
      !type.includes("undefined") &&
      !this.isNType(type)
    );
  }

  private generateConverters(
    types: TypeInfo[],
    functions: FunctionInfo[]
  ): string {
    let output = `// Type converters\n`;

    for (const type of types) {
      output += this.generateConverter(type);
    }

    // Generate function argument converters
    for (const func of functions) {
      if (func.parameters.length > 0) {
        output += this.generateFunctionConverter(func);
      }
    }

    return output;
  }

  private generateConverter(type: TypeInfo): string {
    const converterName = `convert${type.name}`;
    const anchorTypeName = `Anchor${type.name}`;

    if (type.name.endsWith("Args")) {
      return this.generateArgsConverter(type, converterName, anchorTypeName);
    }

    return this.generateObjectConverter(type, converterName, anchorTypeName);
  }

  private generateArgsConverter(
    type: TypeInfo,
    converterName: string,
    anchorTypeName: string
  ): string {
    let output = `export function ${converterName}(\n  args: ${type.name}\n): ${anchorTypeName} {\n`;

    const conversions: string[] = [];

    for (const prop of type.properties) {
      const conversion = this.generatePropertyConversion(
        prop,
        `args.${prop.name}`
      );
      conversions.push(conversion);
    }

    output += `  return [\n    ${conversions.join(",\n    ")}\n  ];\n}\n\n`;
    return output;
  }

  private generateObjectConverter(
    type: TypeInfo,
    converterName: string,
    anchorTypeName: string
  ): string {
    let output = `export function ${converterName}(\n  obj: ${type.name}\n): ${anchorTypeName} {\n`;
    output += `  return {\n`;

    for (const prop of type.properties) {
      const conversion = this.generatePropertyConversion(
        prop,
        `obj.${prop.name}`
      );
      output += `    ${prop.name}: ${conversion},\n`;
    }

    output += `  };\n}\n\n`;
    return output;
  }

  private generateFunctionConverter(func: FunctionInfo): string {
    const converterName = `convert${this.capitalize(func.name)}Args`;
    const argsTypeName = `${this.capitalize(func.name)}Args`;
    const anchorArgsTypeName = `Anchor${this.capitalize(func.name)}Args`;

    let output = `export function ${converterName}(\n  args: ${argsTypeName}\n): ${anchorArgsTypeName} {\n`;

    const conversions: string[] = [];

    for (const param of func.parameters) {
      const propInfo: PropertyInfo = {
        name: param.name,
        type: param.type,
        isOptional: param.isOptional,
        isArray: false,
        isEnum: false,
      };

      const conversion = this.generatePropertyConversion(
        propInfo,
        `args.${param.name}`
      );
      conversions.push(conversion);
    }

    output += `  return [\n    ${conversions.join(",\n    ")}\n  ];\n}\n\n`;
    return output;
  }

  private generatePropertyConversion(
    prop: PropertyInfo,
    accessor: string
  ): string {
    if (prop.isOptional) {
      const innerConversion = this.generateNonOptionalConversion(
        prop,
        accessor
      );
      return `${accessor} !== undefined ? ${innerConversion} : null`;
    }

    return this.generateNonOptionalConversion(prop, accessor);
  }

  private generateNonOptionalConversion(
    prop: PropertyInfo,
    accessor: string
  ): string {
    const baseType = this.getBaseType(prop.type);

    // Handle arrays first
    if (prop.isArray) {
      const elementType = this.extractArrayElementType(prop.type);
      const elementProp: PropertyInfo = {
        ...prop,
        type: elementType,
        isArray: false,
      };
      const elementConversion = this.generateNonOptionalConversion(
        elementProp,
        "item"
      );

      // If the element conversion is just 'item', we can use a direct map
      if (elementConversion === "item") {
        return accessor;
      }

      // For custom types, use the converter function name directly
      if (this.isCustomType(elementType)) {
        return `${accessor}.map(convert${elementType})`;
      }

      return `${accessor}.map(item => ${elementConversion})`;
    }

    // Handle N<T> sized number types in conversions
    const nTypeMatch = this.extractNType(baseType);
    if (nTypeMatch) {
      const size = parseInt(nTypeMatch);
      // N<64> should be converted to anchor.BN
      return size >= 64 ? `new anchor.BN(${accessor})` : accessor;
    }

    switch (baseType) {
      case "number":
        if (
          prop.name.includes("amount") ||
          prop.name.includes("balance") ||
          prop.name.includes("fee") ||
          prop.name.includes("value")
        ) {
          return `new anchor.BN(${accessor})`;
        }
        return accessor;

      case "bigint":
        return `new anchor.BN(${accessor})`;

      case "string":
        if (
          prop.name.includes("address") ||
          prop.name.includes("key") ||
          prop.name.includes("mint") ||
          prop.name.includes("account")
        ) {
          return `new PublicKey(${accessor})`;
        }
        return accessor;

      case "PublicKey":
      case "boolean":
        return accessor;

      default:
        if (this.isCustomType(baseType)) {
          const converterName = `convert${baseType}`;
          return `${converterName}(${accessor})`;
        }

        if (prop.isEnum && prop.enumValues) {
          return `{ [${accessor}.toLowerCase()]: {} }`;
        }

        return accessor;
    }
  }

  private capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }
}
