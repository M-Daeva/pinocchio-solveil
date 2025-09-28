import { OR } from ".";
import {
  BitField,
  Uint16,
  Uint32,
  Uint64,
  Uint128,
  String16,
  String32,
  String64,
  String4096,
} from "../schema/codama";

type IBitFieldArgSet = { flag: boolean; bit?: number };
type IBitFieldArgSetRaw = { value: BitField };

export class TBitField {
  private readonly bitMaxValue: number = 7;
  private readonly byteMaxValue: number = 255;
  private value: number = 0;

  constructor(x?: OR<IBitFieldArgSet, IBitFieldArgSetRaw>) {
    if (!x) {
      this.setRaw(0);
    } else if ("flag" in x) {
      this.set(x.flag, x.bit);
    } else {
      this.setRaw(x.value);
    }
  }

  get(bit: number | undefined = 0): boolean {
    return this.getFlag(bit);
  }

  getRaw(): BitField {
    return this.value;
  }

  set(flag: boolean, bit: number = 0): void {
    this.setFlag(flag, this.validateBit(bit));
  }

  setRaw(x: BitField): void {
    this.value = this.validateByte(x);
  }

  private validateBit(x: number): number {
    if (!Number.isInteger(x) || x < 0 || x > this.bitMaxValue) {
      throw new Error(
        `Value ${x} is not a valid bit (must be integer between 0 and ${this.bitMaxValue})`,
      );
    }
    return x;
  }

  private validateByte(x: number): number {
    if (!Number.isInteger(x) || x < 0 || x > this.byteMaxValue) {
      throw new Error(
        `Value ${x} is not a valid byte (must be integer between 0 and ${this.byteMaxValue})`,
      );
    }
    return x;
  }

  private getFlag(bit: number): boolean {
    return (this.value & (1 << bit)) >> bit == 1;
  }

  private setFlag(flag: boolean, bit: number): void {
    flag ? this.setBit(bit) : this.resetBit(bit);
  }

  private setBit(bit: number): void {
    this.value = this.value | (1 << bit);
  }

  private resetBit(bit: number): void {
    this.value = this.value & ~(1 << bit);
  }
}

export class TUint16 {
  private readonly name: string = "Uint16";
  private readonly byteLength: number = 2;
  private readonly maxValue: number = 0xffff;
  private value: number = 0;

  constructor(x: number | Uint16 = 0) {
    typeof x === "number" ? this.set(x) : this.setRaw(x);
  }

  get(): number {
    return this.value;
  }

  getRaw(): Uint16 {
    return this.uint16ToBytes(this.value);
  }

  set(x: number): void {
    this.value = this.validateUint16(x);
  }

  setRaw(x: Uint16): void {
    if (x.length !== this.byteLength) {
      throw new Error(
        `${this.name} array must have exactly ${this.byteLength} elements`,
      );
    }
    this.value = this.bytesToUint16(x);
  }

  private validateUint16(x: number): number {
    if (!Number.isInteger(x) || x < 0 || x > this.maxValue) {
      throw new Error(
        `Value ${x} is not a valid ${this.name} (must be integer between 0 and ${this.maxValue})`,
      );
    }
    return x;
  }

  private uint16ToBytes(value: number): Uint16 {
    const buffer = new ArrayBuffer(this.byteLength);
    new DataView(buffer).setUint16(0, value, true);
    return Array.from(new Uint8Array(buffer));
  }

  private bytesToUint16(bytes: Uint16): number {
    const buffer = new ArrayBuffer(this.byteLength);
    new Uint8Array(buffer).set(bytes);
    return new DataView(buffer).getUint16(0, true);
  }
}

export class TUint32 {
  private readonly name: string = "Uint32";
  private readonly byteLength: number = 4;
  private readonly maxValue: number = 0xffffffff;
  private value: number = 0;

  constructor(x: number | Uint32 = 0) {
    typeof x === "number" ? this.set(x) : this.setRaw(x);
  }

  get(): number {
    return this.value;
  }

  getRaw(): Uint32 {
    return this.uint32ToBytes(this.value);
  }

  set(x: number): void {
    this.value = this.validateUint32(x);
  }

  setRaw(x: Uint32): void {
    if (x.length !== this.byteLength) {
      throw new Error(
        `${this.name} array must have exactly ${this.byteLength} elements`,
      );
    }
    this.value = this.bytesToUint32(x);
  }

  private validateUint32(x: number): number {
    if (!Number.isInteger(x) || x < 0 || x > this.maxValue) {
      throw new Error(
        `Value ${x} is not a valid ${this.name} (must be integer between 0 and ${this.maxValue})`,
      );
    }
    return x;
  }

  private uint32ToBytes(value: number): Uint32 {
    const buffer = new ArrayBuffer(this.byteLength);
    new DataView(buffer).setUint32(0, value, true);
    return Array.from(new Uint8Array(buffer));
  }

  private bytesToUint32(bytes: Uint32): number {
    const buffer = new ArrayBuffer(this.byteLength);
    new Uint8Array(buffer).set(bytes);
    return new DataView(buffer).getUint32(0, true);
  }
}

export class TUint64 {
  private readonly name: string = "Uint64";
  private readonly byteLength: number = 8;
  private readonly maxValue: bigint = 0xffffffffffffffffn;
  private value: bigint = 0n;

  constructor(x: bigint | Uint64 = 0n) {
    typeof x === "bigint" ? this.set(x) : this.setRaw(x);
  }

  get(): bigint {
    return this.value;
  }

  getRaw(): Uint64 {
    return this.uint64ToBytes(this.value);
  }

  set(x: bigint): void {
    this.value = this.validateUint64(x);
  }

  setRaw(x: Uint64): void {
    if (x.length !== this.byteLength) {
      throw new Error(
        `${this.name} array must have exactly ${this.byteLength} elements`,
      );
    }
    this.value = this.bytesToUint64(x);
  }

  private validateUint64(x: bigint): bigint {
    if (x < 0n || x > this.maxValue) {
      throw new Error(
        `Value ${x} is not a valid ${this.name} (must be between 0 and ${this.maxValue})`,
      );
    }
    return x;
  }

  private uint64ToBytes(value: bigint): Uint64 {
    const buffer = new ArrayBuffer(this.byteLength);
    const view = new DataView(buffer);

    // Split the bigint into two 32-bit parts for little-endian storage
    const low = Number(value & 0xffffffffn);
    const high = Number(value >> 32n);

    view.setUint32(0, low, true);
    view.setUint32(4, high, true);

    return Array.from(new Uint8Array(buffer));
  }

  private bytesToUint64(bytes: Uint64): bigint {
    const buffer = new ArrayBuffer(this.byteLength);
    new Uint8Array(buffer).set(bytes);
    const view = new DataView(buffer);

    // Read two 32-bit values in little-endian format
    const low = BigInt(view.getUint32(0, true));
    const high = BigInt(view.getUint32(4, true));

    // Combine into 64-bit bigint
    return (high << 32n) | low;
  }
}

export class TUint128 {
  private readonly name: string = "Uint128";
  private readonly byteLength: number = 16;
  private readonly maxValue: bigint = 0xffffffffffffffffffffffffffffffffn;
  private value: bigint = 0n;

  constructor(x: bigint | Uint128 = 0n) {
    typeof x === "bigint" ? this.set(x) : this.setRaw(x);
  }

  get(): bigint {
    return this.value;
  }

  getRaw(): Uint128 {
    return this.uint128ToBytes(this.value);
  }

  set(x: bigint): void {
    this.value = this.validateUint128(x);
  }

  setRaw(x: Uint128): void {
    if (x.length !== this.byteLength) {
      throw new Error(
        `${this.name} array must have exactly ${this.byteLength} elements`,
      );
    }
    this.value = this.bytesToUint128(x);
  }

  private validateUint128(x: bigint): bigint {
    if (x < 0n || x > this.maxValue) {
      throw new Error(
        `Value ${x} is not a valid ${this.name} (must be between 0 and ${this.maxValue})`,
      );
    }
    return x;
  }

  private uint128ToBytes(value: bigint): Uint128 {
    const buffer = new ArrayBuffer(this.byteLength);
    const view = new DataView(buffer);

    // Split the bigint into four 32-bit parts for little-endian storage
    const part0 = Number(value & 0xffffffffn);
    const part1 = Number((value >> 32n) & 0xffffffffn);
    const part2 = Number((value >> 64n) & 0xffffffffn);
    const part3 = Number((value >> 96n) & 0xffffffffn);

    view.setUint32(0, part0, true);
    view.setUint32(4, part1, true);
    view.setUint32(8, part2, true);
    view.setUint32(12, part3, true);

    return Array.from(new Uint8Array(buffer));
  }

  private bytesToUint128(bytes: Uint128): bigint {
    const buffer = new ArrayBuffer(this.byteLength);
    new Uint8Array(buffer).set(bytes);
    const view = new DataView(buffer);

    // Read four 32-bit values in little-endian format
    const part0 = BigInt(view.getUint32(0, true));
    const part1 = BigInt(view.getUint32(4, true));
    const part2 = BigInt(view.getUint32(8, true));
    const part3 = BigInt(view.getUint32(12, true));

    // Combine into 128-bit bigint
    return (part3 << 96n) | (part2 << 64n) | (part1 << 32n) | part0;
  }
}

export class TString16 {
  private readonly name: string = "String16";
  private readonly byteLength: number = 16;
  private value: string = "";

  constructor(x: string | String16 = "") {
    typeof x === "string" ? this.set(x) : this.setRaw(x);
  }

  get(): string {
    return this.value;
  }

  getRaw(): String16 {
    return this.string16ToBytes(this.value);
  }

  set(x: string): void {
    const encoded = new TextEncoder().encode(x);
    if (encoded.length > this.byteLength - 1) {
      throw new Error(
        `String too long: ${encoded.length} bytes exceeds maximum of ${this.byteLength - 1} bytes`,
      );
    }
    this.value = x;
  }

  setRaw(x: String16): void {
    if (x.length !== this.byteLength) {
      throw new Error(
        `${this.name} array must have exactly ${this.byteLength} elements`,
      );
    }
    this.value = this.bytesToString(x);
  }

  private string16ToBytes(value: string): String16 {
    const bytes = new TextEncoder().encode(value);

    if (bytes.length > this.byteLength - 1) {
      throw new Error(
        `Encoded string exceeds ${this.byteLength - 1} byte limit`,
      );
    }

    const result = new Array(this.byteLength).fill(0);
    for (let i = 0; i < bytes.length; i++) {
      result[i] = bytes[i];
    }

    return result;
  }

  private bytesToString(bytes: String16): string {
    let length = bytes.findIndex((x) => x === 0);
    length = length === -1 ? bytes.length : length;

    try {
      return new TextDecoder("utf-8", { fatal: true }).decode(
        new Uint8Array(bytes.slice(0, length)),
      );
    } catch (error) {
      throw new Error(`Invalid UTF-8 sequence in byte array: ${error}`);
    }
  }
}

export class TString32 {
  private readonly name: string = "String32";
  private readonly byteLength: number = 32;
  private value: string = "";

  constructor(x: string | String32 = "") {
    typeof x === "string" ? this.set(x) : this.setRaw(x);
  }

  get(): string {
    return this.value;
  }

  getRaw(): String32 {
    return this.stringToBytes(this.value);
  }

  set(x: string): void {
    const encoded = new TextEncoder().encode(x);
    if (encoded.length > this.byteLength - 1) {
      throw new Error(
        `String too long: ${encoded.length} bytes exceeds maximum of ${this.byteLength - 1} bytes`,
      );
    }
    this.value = x;
  }

  setRaw(x: String32): void {
    if (x.length !== this.byteLength) {
      throw new Error(
        `${this.name} array must have exactly ${this.byteLength} elements`,
      );
    }
    this.value = this.bytesToString(x);
  }

  private stringToBytes(value: string): String32 {
    const bytes = new TextEncoder().encode(value);
    if (bytes.length > this.byteLength - 1) {
      throw new Error(
        `Encoded string exceeds ${this.byteLength - 1} byte limit`,
      );
    }
    const result = new Array(this.byteLength).fill(0);
    for (let i = 0; i < bytes.length; i++) {
      result[i] = bytes[i];
    }
    return result;
  }

  private bytesToString(bytes: String32): string {
    let length = bytes.findIndex((x) => x === 0);
    length = length === -1 ? bytes.length : length;
    try {
      return new TextDecoder("utf-8", { fatal: true }).decode(
        new Uint8Array(bytes.slice(0, length)),
      );
    } catch (error) {
      throw new Error(`Invalid UTF-8 sequence in byte array: ${error}`);
    }
  }
}

export class TString64 {
  private readonly name: string = "String64";
  private readonly byteLength: number = 64;
  private value: string = "";

  constructor(x: string | String64 = "") {
    typeof x === "string" ? this.set(x) : this.setRaw(x);
  }

  get(): string {
    return this.value;
  }

  getRaw(): String64 {
    return this.stringToBytes(this.value);
  }

  set(x: string): void {
    const encoded = new TextEncoder().encode(x);
    if (encoded.length > this.byteLength - 1) {
      throw new Error(
        `String too long: ${encoded.length} bytes exceeds maximum of ${this.byteLength - 1} bytes`,
      );
    }
    this.value = x;
  }

  setRaw(x: String64): void {
    if (x.length !== this.byteLength) {
      throw new Error(
        `${this.name} array must have exactly ${this.byteLength} elements`,
      );
    }
    this.value = this.bytesToString(x);
  }

  private stringToBytes(value: string): String64 {
    const bytes = new TextEncoder().encode(value);
    if (bytes.length > this.byteLength - 1) {
      throw new Error(
        `Encoded string exceeds ${this.byteLength - 1} byte limit`,
      );
    }
    const result = new Array(this.byteLength).fill(0);
    for (let i = 0; i < bytes.length; i++) {
      result[i] = bytes[i];
    }
    return result;
  }

  private bytesToString(bytes: String64): string {
    let length = bytes.findIndex((x) => x === 0);
    length = length === -1 ? bytes.length : length;
    try {
      return new TextDecoder("utf-8", { fatal: true }).decode(
        new Uint8Array(bytes.slice(0, length)),
      );
    } catch (error) {
      throw new Error(`Invalid UTF-8 sequence in byte array: ${error}`);
    }
  }
}

export class TString4096 {
  private readonly name: string = "String4096";
  private readonly byteLength: number = 4096;
  private value: string = "";

  constructor(x: string | String4096 = "") {
    typeof x === "string" ? this.set(x) : this.setRaw(x);
  }

  get(): string {
    return this.value;
  }

  getRaw(): String4096 {
    return this.stringToBytes(this.value);
  }

  set(x: string): void {
    const encoded = new TextEncoder().encode(x);
    if (encoded.length > this.byteLength - 1) {
      throw new Error(
        `String too long: ${encoded.length} bytes exceeds maximum of ${this.byteLength - 1} bytes`,
      );
    }
    this.value = x;
  }

  setRaw(x: String4096): void {
    if (x.length !== this.byteLength) {
      throw new Error(
        `${this.name} array must have exactly ${this.byteLength} elements`,
      );
    }
    this.value = this.bytesToString(x);
  }

  private stringToBytes(value: string): String4096 {
    const bytes = new TextEncoder().encode(value);
    if (bytes.length > this.byteLength - 1) {
      throw new Error(
        `Encoded string exceeds ${this.byteLength - 1} byte limit`,
      );
    }
    const result = new Array(this.byteLength).fill(0);
    for (let i = 0; i < bytes.length; i++) {
      result[i] = bytes[i];
    }
    return result;
  }

  private bytesToString(bytes: String4096): string {
    let length = bytes.findIndex((x) => x === 0);
    length = length === -1 ? bytes.length : length;
    try {
      return new TextDecoder("utf-8", { fatal: true }).decode(
        new Uint8Array(bytes.slice(0, length)),
      );
    } catch (error) {
      throw new Error(`Invalid UTF-8 sequence in byte array: ${error}`);
    }
  }
}

export class TEnum<T extends Record<string | number, string | number>> {
  private readonly name: string = "Enum";
  private readonly validValues: number[];
  private value: number = 0;

  constructor(enumObject: T, x: number = 0) {
    this.validValues = Object.values(enumObject).filter(
      (v) => typeof v === "number",
    ) as number[];
    this.setRaw(x);
  }

  get(): number {
    return this.value;
  }

  getRaw(): number {
    return this.value;
  }

  set(x: number): void {
    this.value = this.validateEnum(x);
  }

  setRaw(x: number): void {
    this.value = this.validateEnumValue(x);
  }

  private validateEnum(x: number): number {
    if (!this.validValues.includes(x)) {
      throw new Error(
        `Value ${x} is not a valid enum value for ${this.name}. Valid values: [${this.validValues.join(", ")}]`,
      );
    }
    return x;
  }

  private validateEnumValue(x: number): number {
    if (!Number.isInteger(x) || x < 0 || x > 255) {
      throw new Error(
        `Value ${x} is not a valid 8-bit number (must be integer between 0 and 255)`,
      );
    }
    if (!this.validValues.includes(x)) {
      throw new Error(
        `Value ${x} is not a valid enum value for ${this.name}. Valid values: [${this.validValues.join(", ")}]`,
      );
    }
    return x;
  }
}
