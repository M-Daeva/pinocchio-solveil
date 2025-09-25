import { networks, ProgramName } from "../config";
import {
  Address,
  ProgramDerivedAddressBump,
  ReadonlyUint8Array,
  Rpc,
  RpcDevnet,
  RpcMainnet,
  RpcTransport,
  RpcTransportDevnet,
  RpcTransportMainnet,
  SendAndConfirmTransactionWithSignersFunction,
  SimulateTransactionFunction,
  SolanaRpcApiFromTransport,
} from "gill";

export type Network = (typeof networks)[number];

export type NetworkConfig = {
  [k in Network]: string;
};

export type ProgramAddress = {
  [k in ProgramName]: string;
};

export type RpcMain = RpcMainnet<
  SolanaRpcApiFromTransport<RpcTransportMainnet>
>;
export type RpcDev = RpcDevnet<SolanaRpcApiFromTransport<RpcTransportDevnet>>;
export type RpcLocal = Rpc<SolanaRpcApiFromTransport<RpcTransport>>;

export type RpcAny = RpcMain | RpcDev | RpcLocal;
export type RpcMainOrDev = RpcMain | RpcDev;

export interface ClientAny {
  sendAndConfirmTransaction: SendAndConfirmTransactionWithSignersFunction;
  simulateTransaction: SimulateTransactionFunction;
  rpc: RpcAny;
}

// to get account interface from input and instruction data args interfaces
export type XOR<T, U> = Omit<T, keyof U> & Omit<U, keyof T>;

export type Seed = ReadonlyUint8Array | string;
export type PdaResp = readonly [Address<string>, ProgramDerivedAddressBump];

// Configuration for compute units and priority fees
export interface ComputeConfig {
  priorityFeeMultiplier?: number; // multiplier for base priority fee (default: 1.2)
  priorityFeeBase?: number; // base fee in microlamports to add (default: 0)
  cuMultiplier?: number; // multiplier for simulated CU usage (default: 1.2)
  cuBase?: number; // base CU to add (default: 0)
  maxPriorityFee?: number; // max priority fee cap in microlamports
  minPriorityFee?: number; // min priority fee floor in microlamports
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
