// --- helpers ---
function getFlag(value: number, bit: number): boolean {
  return ((value >> bit) & 1) !== 0;
}

function setFlag(value: number, bit: number, flag: boolean): number {
  if (flag) {
    return value | (1 << bit);
  } else {
    return value & ~(1 << bit);
  }
}

// --- BitField ---
export class BitField {
  private value: number;

  constructor(x: number | boolean = 0) {
    if (typeof x === "boolean") {
      this.value = x ? 1 : 0;
    } else {
      this.value = x & 0xff;
    }
  }

  getRaw(): number {
    return this.value;
  }

  setRaw(x: number): void {
    this.value = x & 0xff;
  }

  getFlag(bit: number): boolean {
    return getFlag(this.value, bit);
  }

  setFlag(bit: number, flag: boolean): void {
    this.value = setFlag(this.value, bit, flag) & 0xff;
  }

  getBit(): boolean {
    return this.getFlag(0);
  }

  setBit(flag: boolean): void {
    this.setFlag(0, flag);
  }
}

// --- Uint16 ---
export class Uint16 {
  private bytes: Uint8Array;

  constructor(x: number | Uint8Array = 0) {
    if (typeof x === "number") {
      this.bytes = new Uint8Array(2);
      new DataView(this.bytes.buffer).setUint16(0, x, true);
    } else {
      if (x.length !== 2) throw new Error("Uint16 requires 2 bytes");
      this.bytes = new Uint8Array(x);
    }
  }

  getRaw(): Uint8Array {
    return new Uint8Array(this.bytes);
  }

  setRaw(x: Uint8Array): void {
    if (x.length !== 2) throw new Error("Uint16 requires 2 bytes");
    this.bytes.set(x);
  }

  get(): number {
    return new DataView(this.bytes.buffer).getUint16(0, true);
  }

  set(x: number): void {
    new DataView(this.bytes.buffer).setUint16(0, x, true);
  }

  // to support codama types
  toArray(): Array<number> {
    return Array.from(this.bytes);
  }
}

// --- Uint32 ---
export class Uint32 {
  private bytes: Uint8Array;

  constructor(x: number | Uint8Array = 0) {
    if (typeof x === "number") {
      this.bytes = new Uint8Array(4);
      new DataView(this.bytes.buffer).setUint32(0, x, true);
    } else {
      if (x.length !== 4) throw new Error("Uint32 requires 4 bytes");
      this.bytes = new Uint8Array(x);
    }
  }

  getRaw(): Uint8Array {
    return new Uint8Array(this.bytes);
  }

  setRaw(x: Uint8Array): void {
    if (x.length !== 4) throw new Error("Uint32 requires 4 bytes");
    this.bytes.set(x);
  }

  get(): number {
    return new DataView(this.bytes.buffer).getUint32(0, true);
  }

  set(x: number): void {
    new DataView(this.bytes.buffer).setUint32(0, x, true);
  }

  // to support codama types
  toArray(): Array<number> {
    return Array.from(this.bytes);
  }
}

// --- Uint64 ---
export class Uint64 {
  private bytes: Uint8Array;

  constructor(x: bigint | Uint8Array = 0n) {
    if (typeof x === "bigint") {
      this.bytes = new Uint8Array(8);
      new DataView(this.bytes.buffer).setBigUint64(0, x, true);
    } else {
      if (x.length !== 8) throw new Error("Uint64 requires 8 bytes");
      this.bytes = new Uint8Array(x);
    }
  }

  getRaw(): Uint8Array {
    return new Uint8Array(this.bytes);
  }

  setRaw(x: Uint8Array): void {
    if (x.length !== 8) throw new Error("Uint64 requires 8 bytes");
    this.bytes.set(x);
  }

  get(): bigint {
    return new DataView(this.bytes.buffer).getBigUint64(0, true);
  }

  set(x: bigint): void {
    new DataView(this.bytes.buffer).setBigUint64(0, x, true);
  }

  // to support codama types
  toArray(): Array<number> {
    return Array.from(this.bytes);
  }
}

// --- Uint128 ---
export class Uint128 {
  private bytes: Uint8Array;

  constructor(x: bigint | Uint8Array = 0n) {
    if (typeof x === "bigint") {
      this.bytes = new Uint8Array(16);
      let tmp = x;
      for (let i = 0; i < 16; i++) {
        this.bytes[i] = Number(tmp & 0xffn);
        tmp >>= 8n;
      }
    } else {
      if (x.length !== 16) throw new Error("Uint128 requires 16 bytes");
      this.bytes = new Uint8Array(x);
    }
  }

  getRaw(): Uint8Array {
    return new Uint8Array(this.bytes);
  }

  setRaw(x: Uint8Array): void {
    if (x.length !== 16) throw new Error("Uint128 requires 16 bytes");
    this.bytes.set(x);
  }

  get(): bigint {
    let result = 0n;
    for (let i = 15; i >= 0; i--) {
      result = (result << 8n) | BigInt(this.bytes[i]);
    }
    return result;
  }

  set(x: bigint): void {
    let tmp = x;
    for (let i = 0; i < 16; i++) {
      this.bytes[i] = Number(tmp & 0xffn);
      tmp >>= 8n;
    }
  }

  // to support codama types
  toArray(): Array<number> {
    return Array.from(this.bytes);
  }
}
