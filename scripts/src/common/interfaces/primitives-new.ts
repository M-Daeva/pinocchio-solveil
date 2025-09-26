import { Uint32, Uint64, Uint128 } from "../schema/codama";

export class IUint32 {
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
    if (x.length !== 4) {
      throw new Error("Uint32 array must have exactly 4 elements");
    }
    this.value = this.bytesToUint32(x);
  }

  private validateUint32(x: number): number {
    if (!Number.isInteger(x) || x < 0 || x > 0xffffffff) {
      throw new Error(
        `Value ${x} is not a valid uint32 (must be integer between 0 and 4294967295)`,
      );
    }
    return x;
  }

  private uint32ToBytes(value: number): Uint32 {
    const buffer = new ArrayBuffer(4);
    new DataView(buffer).setUint32(0, value, true);
    return Array.from(new Uint8Array(buffer));
  }

  private bytesToUint32(bytes: Uint32): number {
    const buffer = new ArrayBuffer(4);
    new Uint8Array(buffer).set(bytes);
    return new DataView(buffer).getUint32(0, true);
  }
}

export class IUint64 {
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
    if (x.length !== 8) {
      throw new Error("Uint64 array must have exactly 8 elements");
    }
    this.value = this.bytesToUint64(x);
  }

  private validateUint64(x: bigint): bigint {
    if (x < 0n || x > 0xffffffffffffffffn) {
      throw new Error(
        `Value ${x} is not a valid uint64 (must be between 0 and 18446744073709551615)`,
      );
    }
    return x;
  }

  private uint64ToBytes(value: bigint): Uint64 {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);

    // Split the bigint into two 32-bit parts for little-endian storage
    const low = Number(value & 0xffffffffn);
    const high = Number(value >> 32n);

    view.setUint32(0, low, true);
    view.setUint32(4, high, true);

    return Array.from(new Uint8Array(buffer));
  }

  private bytesToUint64(bytes: Uint64): bigint {
    const buffer = new ArrayBuffer(8);
    new Uint8Array(buffer).set(bytes);
    const view = new DataView(buffer);

    // Read two 32-bit values in little-endian format
    const low = BigInt(view.getUint32(0, true));
    const high = BigInt(view.getUint32(4, true));

    // Combine into 64-bit bigint
    return (high << 32n) | low;
  }
}

export class IUint128 {
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
    if (x.length !== 16) {
      throw new Error("Uint128 array must have exactly 16 elements");
    }
    this.value = this.bytesToUint128(x);
  }

  private validateUint128(x: bigint): bigint {
    if (x < 0n || x > 0xffffffffffffffffffffffffffffffffn) {
      throw new Error(
        `Value ${x} is not a valid uint128 (must be between 0 and 340282366920938463463374607431768211455)`,
      );
    }
    return x;
  }

  private uint128ToBytes(value: bigint): Uint128 {
    const buffer = new ArrayBuffer(16);
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
    const buffer = new ArrayBuffer(16);
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
