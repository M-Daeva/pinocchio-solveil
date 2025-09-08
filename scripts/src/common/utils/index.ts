import { AES, enc } from "crypto-js";
import util from "util";
import { all, create } from "mathjs";
import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Network, TxParams } from "../interfaces";
import { getSimulationComputeUnits } from "@solana-developers/helpers";
import { NETWORK_CONFIG } from "../config";
import axios, {
  AxiosRequestConfig,
  AxiosInstance,
  CreateAxiosDefaults,
} from "axios";
import {
  AddressLookupTableAccount,
  ComputeBudgetProgram,
  Connection,
  PublicKey,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";

export const DECIMAL_PLACES = 18;

export const l = console.log.bind(console);

export function li(object: any) {
  console.log(
    util.inspect(object, {
      showHidden: false,
      depth: null,
      colors: true,
    })
  );
}

export function logAndReturn<T>(object: T, isDisplayed: boolean = false): T {
  if (isDisplayed) {
    l();
    li(object);
    l();
  }
  return object;
}

export function floor(num: number, digits: number = 0): number {
  const k = 10 ** digits;
  return Math.floor(k * num) / k;
}

export function round(num: number, digits: number = 0): number {
  const k = 10 ** digits;
  return Math.round(k * num) / k;
}

export function getLast<T>(arr: T[]): T | undefined {
  return arr[arr.length - 1];
}

export function dedupVector<T>(arr: T[]): T[] {
  return Array.from(new Set(arr));
}

export function getTimestamp(): string {
  return Date.now().toString();
}

export function getLocalBlockTime(): number {
  return floor(Date.now() / 1e3);
}

// blockTimeOffset = contractBlockTime - localBlockTime
export function getBlockTime(blockTimeOffset: number): number {
  return blockTimeOffset + getLocalBlockTime();
}

export async function wait(delayInMilliseconds: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, delayInMilliseconds);
  });
}

export class Request {
  private req: AxiosInstance;

  constructor(config: CreateAxiosDefaults = {}) {
    this.req = axios.create(config);
  }

  async get<T>(url: string, config?: Object): Promise<T> {
    return (await this.req.get(url, config)).data;
  }

  async post(url: string, params: Object, config?: AxiosRequestConfig) {
    return (await this.req.post(url, params, config)).data;
  }
}

export function encrypt(data: string, key: string): string {
  return AES.encrypt(data, key).toString();
}

export function decrypt(
  encryptedData: string,
  key: string
): string | undefined {
  // "Malformed UTF-8 data" workaround
  try {
    const bytes = AES.decrypt(encryptedData, key);
    return bytes.toString(enc.Utf8);
  } catch (error) {
    return;
  }
}

export function getPaginationAmount(
  maxPaginationAmount: number,
  maxCount: number
): number {
  // limit maxPaginationAmount
  maxPaginationAmount = Math.min(
    maxPaginationAmount,
    maxCount || maxPaginationAmount
  );

  // update maxPaginationAmount to balance the load
  return maxCount
    ? Math.ceil(maxCount / Math.ceil(maxCount / maxPaginationAmount))
    : maxPaginationAmount;
}

// configure the default type of numbers as BigNumbers
const math = create(all, {
  // Default type of number
  // Available options: 'number' (default), 'BigNumber', or 'Fraction'
  number: "BigNumber",
  // Number of significant digits for BigNumbers
  precision: 256,
});

export function numberFrom(
  value: number | string | bigint | undefined | null
): math.BigNumber {
  if (typeof value === "undefined" || value === "") {
    return math.bignumber(0);
  }

  return typeof value === "bigint"
    ? math.bignumber(value.toString())
    : math.bignumber(value);
}

export function decimalFrom(value: math.BigNumber): string {
  return value.toPrecision(DECIMAL_PLACES);
}

export function publicKeyFromString(publicKey: anchor.web3.PublicKey | string) {
  return typeof publicKey === "string" ? new PublicKey(publicKey) : publicKey;
}

export function getProgram<IDL extends anchor.Idl = anchor.Idl>(
  provider: anchor.AnchorProvider,
  idl: IDL
): anchor.Program<IDL> {
  return new anchor.Program<IDL>(idl, provider);
}

export function getRpc(network: Network): string {
  return NETWORK_CONFIG[network];
}

export function getProvider(
  wallet: anchor.Wallet,
  rpc: string,
  commitment: anchor.web3.Commitment
): anchor.AnchorProvider {
  const connection = new Connection(rpc, commitment);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment,
  });
  anchor.setProvider(provider);

  return provider;
}

export async function handleTx(
  provider: anchor.AnchorProvider,
  instructions: TransactionInstruction[],
  params: TxParams
): Promise<anchor.web3.TransactionSignature> {
  const { connection, wallet } = provider;

  let { lookupTables, priorityFee, cpu, signers } = params;
  lookupTables = lookupTables || [];
  priorityFee = { k: priorityFee?.k || 1, b: priorityFee?.b || 0 };
  cpu = { k: cpu?.k || 1, b: cpu?.b || 0 };
  signers = signers || []; // additional signers (like mint keypairs)

  // TODO: check this option: https://www.helius.dev/docs/priority-fee/estimating-fees-using-serialized-transaction
  // get priority fees
  // https://solana.com/developers/guides/advanced/how-to-use-priority-fees#how-do-i-estimate-priority-fees
  const prioritizationFees = await connection.getRecentPrioritizationFees({
    lockedWritableAccounts: [wallet.publicKey],
  });

  const defaultPriorityFee = prioritizationFees.length
    ? Math.ceil(
        prioritizationFees.reduce(
          (acc, cur) => acc + cur.prioritizationFee,
          0
        ) / prioritizationFees.length
      )
    : 0;

  // https://solana.com/developers/guides/advanced/how-to-request-optimal-compute#how-to-request-compute-budget
  let [microLamports, units, { blockhash, lastValidBlockHeight }] =
    await Promise.all([
      priorityFee.k * defaultPriorityFee + priorityFee.b,
      getSimulationComputeUnits(
        connection,
        instructions,
        wallet.publicKey,
        lookupTables
      ),
      connection.getLatestBlockhash(),
    ]);

  instructions.unshift(
    ComputeBudgetProgram.setComputeUnitPrice({ microLamports })
  );

  units = cpu.k * (units || 0) + cpu.b;
  if (units) {
    // probably should add some margin of error to units
    instructions.unshift(ComputeBudgetProgram.setComputeUnitLimit({ units }));
  }

  // TODO .compileToV0Message(lookupTables)
  // create transaction message
  const message = new TransactionMessage({
    instructions,
    recentBlockhash: blockhash,
    payerKey: wallet.publicKey,
  }).compileToLegacyMessage();

  // create versioned transaction
  const transaction = new VersionedTransaction(message);

  // sign with additional signers first (like mint keypairs)
  if (signers.length > 0) {
    transaction.sign(signers);
  }

  // then sign with the wallet
  const signedTx = await wallet.signTransaction(transaction);

  // send transaction
  const signature = await connection.sendTransaction(signedTx);

  await connection.confirmTransaction({
    blockhash,
    lastValidBlockHeight,
    signature,
  });

  return signature;
}

export function getHandleTx(provider: anchor.AnchorProvider) {
  return async (
    instructions: TransactionInstruction[],
    params: TxParams,
    isDisplayed: boolean
  ): Promise<anchor.web3.TransactionSignature> => {
    const tx = await handleTx(provider, instructions, params);
    return logAndReturn(tx, isDisplayed);
  };
}

export async function getOrCreateAtaInstructions(
  connection: anchor.web3.Connection,
  payer: PublicKey,
  mintPubkey: PublicKey,
  ownerPubkey: PublicKey,
  allowOwnerOffCurve: boolean
): Promise<{
  ata: anchor.web3.PublicKey;
  ixs: anchor.web3.TransactionInstruction[];
}> {
  // calculate the ATA address
  const associatedToken = await spl.getAssociatedTokenAddress(
    mintPubkey,
    ownerPubkey,
    allowOwnerOffCurve,
    spl.TOKEN_PROGRAM_ID,
    spl.ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // check if the account exists and is properly initialized
  try {
    await spl.getAccount(
      connection,
      associatedToken,
      undefined,
      spl.TOKEN_PROGRAM_ID
    );

    // account exists and is properly initialized
    return {
      ata: associatedToken,
      ixs: [],
    };
  } catch (_) {
    // create the ATA creation instruction
    const instruction = spl.createAssociatedTokenAccountInstruction(
      payer,
      associatedToken,
      ownerPubkey,
      mintPubkey,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID
    );

    return {
      ata: associatedToken,
      ixs: [instruction],
    };
  }
}

export function getTokenProgramFactory(provider: anchor.AnchorProvider) {
  return async (mint: anchor.web3.PublicKey) => {
    // check if it's SOL (represented by PublicKey.default)
    if (mint.equals(PublicKey.default)) {
      throw new Error(`Mint ${mint.toString()} represents Sol`);
    }

    // it's a token, so get the mint account to determine which token program owns it
    const mintAccount = await provider.connection.getAccountInfo(mint);
    if (!mintAccount) {
      throw new Error(`Mint account ${mint.toString()} not found`);
    }

    // determine if it's Token Program or Token 2022
    let tokenProgram: PublicKey;
    if (mintAccount.owner.equals(spl.TOKEN_PROGRAM_ID)) {
      tokenProgram = spl.TOKEN_PROGRAM_ID;
    } else if (mintAccount.owner.equals(spl.TOKEN_2022_PROGRAM_ID)) {
      tokenProgram = spl.TOKEN_2022_PROGRAM_ID;
    } else {
      throw new Error(`Unknown token program: ${mintAccount.owner.toString()}`);
    }

    return tokenProgram;
  };
}

type RustIntType = "u8" | "u16" | "u32" | "u64" | "u128";

/**
 * Converts a TypeScript number to a Buffer based on the specified Rust integer type
 * @param value - The number to convert
 * @param rustType - The Rust integer type (u8, u16, u32, u64, u128)
 * @returns Buffer representing the number in little-endian format
 * @throws Error if the value exceeds the maximum for the specified type
 */
export function numberToRustBuffer(
  value: number,
  rustType: RustIntType
): Buffer {
  // Validate input bounds for each type
  const maxValues = {
    u8: 255, // 2^8 - 1
    u16: 65535, // 2^16 - 1
    u32: 4294967295, // 2^32 - 1
    u64: BigInt("18446744073709551615"), // 2^64 - 1 (as bigint)
    u128: BigInt("340282366920938463463374607431768211455"), // 2^128 - 1 (as bigint)
  };

  // Byte sizes for each type
  const byteSizes = {
    u8: 1,
    u16: 2,
    u32: 4,
    u64: 8,
    u128: 16,
  };

  // Handle u8 specially - simple array conversion
  if (rustType === "u8") {
    if (value < 0 || value > maxValues.u8 || !Number.isInteger(value)) {
      throw new Error(
        `Value ${value} is out of range for u8 (0-${maxValues.u8})`
      );
    }
    return Buffer.from([value]);
  }

  // Handle u16, u32 with anchor.BN for consistency
  if (rustType === "u16" || rustType === "u32") {
    const maxValue = maxValues[rustType] as number;
    if (value < 0 || value > maxValue || !Number.isInteger(value)) {
      throw new Error(
        `Value ${value} is out of range for ${rustType} (0-${maxValue})`
      );
    }

    return new anchor.BN(value).toArrayLike(Buffer, "le", byteSizes[rustType]);
  }

  // Handle u64 and u128 - these require bigint for full range support
  if (rustType === "u64" || rustType === "u128") {
    // Convert number to bigint for large value handling
    let bigintValue: bigint;

    if (typeof value === "bigint") {
      bigintValue = value;
    } else {
      if (!Number.isInteger(value) || value < 0) {
        throw new Error(
          `Value ${value} must be a non-negative integer for ${rustType}`
        );
      }
      bigintValue = BigInt(value);
    }

    const maxValue = maxValues[rustType] as bigint;
    if (bigintValue < BigInt(0) || bigintValue > maxValue) {
      throw new Error(
        `Value ${bigintValue} is out of range for ${rustType} (0-${maxValue})`
      );
    }

    // For very large numbers, we need to handle them as bigint
    return new anchor.BN(bigintValue.toString()).toArrayLike(
      Buffer,
      "le",
      byteSizes[rustType]
    );
  }

  throw new Error(`Unsupported Rust type: ${rustType}`);
}
