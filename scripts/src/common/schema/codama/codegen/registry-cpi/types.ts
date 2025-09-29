import { Address } from "gill";

export interface IAssetItem {
  amount: bigint;
  asset: Address;
}

export interface IRange {
  min: number;
  max: number;
}

export interface IStructWithArray {
  list: IIAssetItem[];
}
