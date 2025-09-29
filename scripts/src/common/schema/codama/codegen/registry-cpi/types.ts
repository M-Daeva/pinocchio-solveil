import { Address } from "gill";

export interface IAssetItem {
  amount: bigint;
  asset: Address;
}

export interface IRange {
  min: number;
  max: number;
}

export enum ITargetEnum {
  Spl,
  Proxy,
  Route,
}

export interface IStructWithArray {
  list: IAssetItem[];
}
