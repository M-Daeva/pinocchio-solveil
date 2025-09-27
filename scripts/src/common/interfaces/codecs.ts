import { Address } from "gill";
import { SYSTEM_PROGRAM_ADDRESS } from "gill/programs";
import { TBitField, TUint32, TUint64 } from "./primitives";
import {
  Config,
  Range,
  AssetItem,
  UpdateConfigInstructionDataArgs,
} from "../schema/codama";

const DEFAULT_ADDRESS = SYSTEM_PROGRAM_ADDRESS;

// should be generated from contract code
export interface IAssetItem {
  amount: bigint;
  asset: Address;
}

export function encAssetItem(x?: IAssetItem): AssetItem {
  return {
    amount: new TUint64(x?.amount).getRaw(),
    asset: x?.asset || DEFAULT_ADDRESS,
  };
}

export function decAssetItem(x?: AssetItem): IAssetItem {
  return {
    amount: new TUint64(x?.amount).get(),
    asset: x?.asset || DEFAULT_ADDRESS,
  };
}

export interface IRange {
  min: number;
  max: number;
}

export function encRange(x?: IRange): Range {
  return {
    min: new TUint32(x?.min).getRaw(),
    max: new TUint32(x?.max).getRaw(),
  };
}

export function decRange(x?: Range): IRange {
  return {
    min: new TUint32(x?.min).get(),
    max: new TUint32(x?.max).get(),
  };
}

export interface IConfig {
  admin: Address;
  isPaused: boolean;
  rotationTimeout: number;
  registrationFee: IAssetItem;
  dataSizeRange: IRange;
}

// type/state codecs args should be optional
export function encConfig(x?: IConfig): Config {
  return {
    admin: x?.admin || DEFAULT_ADDRESS,
    isPaused: new TBitField({ flag: x?.isPaused || false }).getRaw(),
    rotationTimeout: new TUint32(x?.rotationTimeout).getRaw(),
    registrationFee: encAssetItem(x?.registrationFee),
    dataSizeRange: encRange(x?.dataSizeRange),
  };
}

export function decConfig(x?: Config): IConfig {
  return {
    admin: x?.admin || DEFAULT_ADDRESS,
    isPaused: new TBitField({ value: x?.isPaused || 0 }).get(),
    rotationTimeout: new TUint32(x?.rotationTimeout).get(),
    registrationFee: decAssetItem(x?.registrationFee),
    dataSizeRange: decRange(x?.dataSizeRange),
  };
}

export interface IUpdateConfigInstructionDataArgs {
  admin?: Address;
  is_paused?: boolean;
  rotation_timeout?: number;
  registration_fee_amount?: bigint;
  data_size_range?: IRange;
}

// we need only encoders for ixs
// ix codecs args can't be optional
export function encUpdateConfigInstructionDataArgs(
  x: IUpdateConfigInstructionDataArgs,
): UpdateConfigInstructionDataArgs {
  const ft = new FlagTracker();

  let data: UpdateConfigInstructionDataArgs = {
    flags: 0,
    admin: ft.add(x.admin, DEFAULT_ADDRESS),
    isPaused: new TBitField({ flag: ft.add(x.is_paused, false) }).getRaw(),
    rotationTimeout: new TUint32(ft.add(x.rotation_timeout)).getRaw(),
    registrationFeeAmount: new TUint64(
      ft.add(x.registration_fee_amount, 0n),
    ).getRaw(),
    dataSizeRange: encRange(ft.add(x.data_size_range)),
  };

  // flags should be updated based on input data fields presence
  return { ...data, flags: ft.get() };
}

export class FlagTracker {
  private counter: number = 0;
  private mask: number[] = [];

  constructor(counter: number = 0) {
    this.counter = counter;
  }

  add<T>(x: T | undefined, defaultValue?: T): T {
    this.counter++;

    if (typeof x !== "undefined") {
      this.mask.push(this.counter - 1);
      return x;
    }

    if (defaultValue) {
      return defaultValue;
    }

    return undefined as T;
  }

  get(): number {
    let flags = new TBitField();
    for (const bit of this.mask) {
      flags.set(true, bit);
    }

    return flags.getRaw();
  }
}
