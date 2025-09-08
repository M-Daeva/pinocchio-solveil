import { AddressLookupTableAccount, Keypair, PublicKey } from "@solana/web3.js";
import { networks, ProgramName } from "../config";

export type Network = (typeof networks)[number];

export type NetworkConfig = {
  [k in Network]: string;
};

export type ProgramAddress = {
  [k in ProgramName]: string;
};

/**
 * y = k * x + b
 */
export interface LinearParams {
  k: number;
  b: number;
}

export interface TxParams {
  lookupTables?: AddressLookupTableAccount[];
  priorityFee?: LinearParams;
  cpu?: LinearParams;
  signers?: Keypair[];
}

type D = 8 | 16 | 32 | 64;
export type N<T extends D> = number;

export interface Backup {
  nonce: number;
  data: string;
}

export interface DataRecord {
  Date: number;
  label: string;
  password: string;
  note: string;
}
