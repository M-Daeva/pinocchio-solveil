import { Uint32 } from "../schema/codama";

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
    let buffer = new ArrayBuffer(4);
    new DataView(buffer).setUint32(0, value, true);
    return Array.from(new Uint8Array(buffer));
  }

  private bytesToUint32(bytes: Uint32): number {
    let buffer = new ArrayBuffer(4);
    new Uint8Array(buffer).set(bytes);
    return new DataView(buffer).getUint32(0, true);
  }
}
