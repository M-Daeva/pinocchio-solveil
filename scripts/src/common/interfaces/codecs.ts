import { Address } from "gill";
import { Config, Range, AssetItem } from "../schema/codama";
import { TBitField, TUint32, TUint64 } from "./primitives";

// should be generated from contract code
export interface IAssetItem {
  amount: bigint;
  asset: Address;
}

export function encAssetItem(x: IAssetItem): AssetItem {
  return {
    amount: new TUint64(x.amount).getRaw(),
    asset: x.asset,
  };
}

export function decAssetItem(x: AssetItem): IAssetItem {
  return {
    amount: new TUint64(x.amount).get(),
    asset: x.asset,
  };
}

export interface IRange {
  min: number;
  max: number;
}

export function encRange(x: IRange): Range {
  return {
    min: new TUint32(x.min).getRaw(),
    max: new TUint32(x.max).getRaw(),
  };
}

export function decRange(x: Range): IRange {
  return {
    min: new TUint32(x.min).get(),
    max: new TUint32(x.max).get(),
  };
}

export interface IConfig {
  admin: Address;
  isPaused: boolean;
  rotationTimeout: number;
  registrationFee: IAssetItem;
  dataSizeRange: IRange;
}

export function encConfig(x: IConfig): Config {
  return {
    admin: x.admin,
    isPaused: new TBitField({ flag: x.isPaused }).getRaw(),
    rotationTimeout: new TUint32(x.rotationTimeout).getRaw(),
    registrationFee: encAssetItem(x.registrationFee),
    dataSizeRange: encRange(x.dataSizeRange),
  };
}

export function decConfig(x: Config): IConfig {
  return {
    admin: x.admin,
    isPaused: new TBitField({ value: x.isPaused }).get(),
    rotationTimeout: new TUint32(x.rotationTimeout).get(),
    registrationFee: decAssetItem(x.registrationFee),
    dataSizeRange: decRange(x.dataSizeRange),
  };
}
