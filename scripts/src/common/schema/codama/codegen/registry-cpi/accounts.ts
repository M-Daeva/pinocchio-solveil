import { Address } from "gill";

export interface IBump {
  config: number;
  user_counter: number;
  rotation_state: number;
}

export interface IConfig {
  admin: Address;
  rotation_timeout: number;
  registration_fee: AssetItem;
  data_size_range: Range;
}

export interface IUserCounter {
  last_user_id: number;
}

export interface IRotationState {
  owner: Address;
  new_owner: Address;
  expiration_date: bigint;
}

export interface IUserId {
  id: number;
  account_bump: number;
  rotation_state_bump: number;
}

export interface IUserAccount {
  data: string;
  nonce: bigint;
  max_size: number;
}
