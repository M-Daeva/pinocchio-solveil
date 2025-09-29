import { Address } from "gill";
import { IAssetItem, IRange } from "./types";

export interface IBump {
  config: number;
  userCounter: number;
  rotationState: number;
}

export interface IConfig {
  admin: Address;
  isPaused: boolean;
  rotationTimeout: number;
  registrationFee: IAssetItem;
  dataSizeRange: IRange;
}

export interface IUserCounter {
  lastUserId: number;
}

export interface IRotationState {
  owner: Address;
  newOwner: Address;
  expirationDate: bigint;
}

export interface IUserId {
  flags: boolean;
  id: number;
  accountBump: number;
  rotationStateBump: number;
}

export interface IUserAccount {
  data: string;
  nonce: bigint;
  maxSize: number;
}
