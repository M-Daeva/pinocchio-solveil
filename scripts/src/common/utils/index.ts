import { AES, enc } from "crypto-js";
import util from "util";
import { all, create } from "mathjs";
import { COMMITMENT, NETWORK_CONFIG } from "../config";
import { BN } from "bn.js";
import axios, {
  AxiosRequestConfig,
  AxiosInstance,
  CreateAxiosDefaults,
} from "axios";
import {
  ClientAny,
  ComputeConfig,
  Ix,
  Network,
  PdaResp,
  RpcAny,
  Seed,
} from "../interfaces";
import {
  getAssociatedTokenAccountAddress,
  getCreateAssociatedTokenInstruction,
  getSetComputeUnitLimitInstruction,
  getSetComputeUnitPriceInstruction,
  SYSTEM_PROGRAM_ADDRESS,
  TOKEN_2022_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
} from "gill/programs";
import {
  Address,
  compileTransaction,
  createSolanaClient,
  createSolanaRpc,
  createTransaction,
  devnet,
  DevnetUrl,
  getProgramDerivedAddress,
  KeyPairSigner,
  LAMPORTS_PER_SOL,
  localnet,
  LocalnetUrl,
  mainnet,
  MainnetUrl,
  signTransaction,
  SolanaClient,
} from "gill";

export const DECIMAL_PLACES = 18;

export const l = console.log.bind(console);

export function li(object: any) {
  console.log(
    util.inspect(object, {
      showHidden: false,
      depth: null,
      colors: true,
    }),
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
  key: string,
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
  maxCount: number,
): number {
  // limit maxPaginationAmount
  maxPaginationAmount = Math.min(
    maxPaginationAmount,
    maxCount || maxPaginationAmount,
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
  value: number | string | bigint | undefined | null,
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

export function getRpc(network: Network): RpcAny {
  const url = NETWORK_CONFIG[network];

  if (network === "MAINNET") {
    return createSolanaRpc(mainnet(url));
  } else if (network === "DEVNET") {
    return createSolanaRpc(devnet(url));
  } else {
    return createSolanaRpc(localnet(url));
  }
}

export function getClient(network: Network): ClientAny {
  const url = { urlOrMoniker: NETWORK_CONFIG[network] };

  let client:
    | SolanaClient<MainnetUrl>
    | SolanaClient<DevnetUrl>
    | SolanaClient<LocalnetUrl>;

  if (network === "MAINNET") {
    client = createSolanaClient<MainnetUrl>(url);
  } else if (network === "DEVNET") {
    client = createSolanaClient<DevnetUrl>(url);
  } else {
    client = createSolanaClient<LocalnetUrl>(url);
  }

  return {
    sendAndConfirmTransaction: client.sendAndConfirmTransaction,
    simulateTransaction: client.simulateTransaction,
    rpc: client.rpc,
  };
}

export async function handleTx(
  client: ClientAny,
  sender: KeyPairSigner,
  signerList: CryptoKeyPair[],
  instructions: Ix[],
  computeConfig: ComputeConfig = {},
  isDisplayed: boolean = false,
) {
  // Get latest blockhash for transaction
  const { value: latestBlockhash } = await client.rpc
    .getLatestBlockhash()
    .send();

  // Create transaction for simulation
  const simulationTx = createTransaction({
    version: "legacy",
    feePayer: sender.address,
    latestBlockhash,
    instructions,
  });

  // Get optimal priority fee and compute units in parallel
  const [optimalPriorityFee, optimalComputeUnits] = await Promise.all([
    getOptimalPriorityFee(client.rpc, sender.address, computeConfig),
    getOptimalComputeUnits(
      client.simulateTransaction,
      simulationTx,
      computeConfig,
    ),
  ]);

  // Build final instructions array with compute budget instructions
  let finalInstructions = [];

  // Add compute unit price instruction (priority fee)
  if (optimalPriorityFee > 0) {
    finalInstructions.push(
      getSetComputeUnitPriceInstruction({
        microLamports: optimalPriorityFee,
      }),
    );
  }

  // Add compute unit limit instruction
  if (optimalComputeUnits) {
    finalInstructions.push(
      getSetComputeUnitLimitInstruction({
        units: optimalComputeUnits,
      }),
    );
  }

  // Add the main instruction
  finalInstructions = [...finalInstructions, ...instructions];

  // Create final transaction with all instructions
  const finalTx = createTransaction({
    version: simulationTx.version,
    feePayer: simulationTx.feePayer.address,
    latestBlockhash: simulationTx.lifetimeConstraint,
    instructions: finalInstructions,
  });

  // Sign and send transaction
  const signedTransaction = await signTransaction(
    signerList,
    compileTransaction(finalTx),
  );

  const signature = await client.sendAndConfirmTransaction(signedTransaction, {
    commitment: COMMITMENT,
  });
  const txResponse = await client.rpc.getTransaction(signature).send();

  return logAndReturn(txResponse, isDisplayed);
}

/**
 * Get recent prioritization fees for calculating optimal priority fee
 */
async function getOptimalPriorityFee(
  rpc: RpcAny,
  payerAddress: Address,
  config: ComputeConfig = {},
): Promise<number> {
  const PRIORITY_FEE_MULTIPLIER = 1.0;
  const PRIORITY_FEE_BASE = 0;
  const MAX_PRIORITY_FEE = 0.01 * LAMPORTS_PER_SOL;
  const MIN_PRIORITY_FEE = 0;

  try {
    // Get recent prioritization fees
    const prioritizationFees = await rpc
      .getRecentPrioritizationFees([payerAddress])
      .send();

    let basePriorityFee = 0;
    if (prioritizationFees && prioritizationFees.length > 0) {
      // Calculate average
      const feeSum = prioritizationFees.reduce(
        (acc, cur) => acc + cur.prioritizationFee,
        BigInt(0),
      );
      const feeAvg = feeSum / BigInt(prioritizationFees.length);

      basePriorityFee = Math.ceil(Number(feeAvg));
    }

    // Apply multiplier and base adjustment
    const {
      priorityFeeMultiplier = PRIORITY_FEE_MULTIPLIER,
      priorityFeeBase = PRIORITY_FEE_BASE,
      maxPriorityFee = MAX_PRIORITY_FEE,
      minPriorityFee = MIN_PRIORITY_FEE,
    } = config;

    let finalPriorityFee = Math.ceil(
      basePriorityFee * priorityFeeMultiplier + priorityFeeBase,
    );

    // Apply min/max bounds
    finalPriorityFee = Math.max(minPriorityFee, finalPriorityFee);
    finalPriorityFee = Math.min(maxPriorityFee, finalPriorityFee);

    return finalPriorityFee;
  } catch (error) {
    console.warn("Failed to get recent prioritization fees:", error);
    return config.priorityFeeBase || 0;
  }
}

/**
 * Simulate transaction to get compute units and calculate optimal CU limit
 */
async function getOptimalComputeUnits(
  simulateTransaction: any,
  transaction: any,
  config: ComputeConfig = {},
): Promise<number | null> {
  const CU_MULTIPLIER = 1.1;
  const CU_BASE = 0;

  try {
    const simulationResult = await simulateTransaction(transaction);
    const unitsConsumed = simulationResult.value?.unitsConsumed;

    if (!unitsConsumed) {
      console.warn("No compute units consumed in simulation");
      return null;
    }

    // Convert BigInt to Number safely for calculation
    const unitsConsumedNum =
      typeof unitsConsumed === "bigint" ? Number(unitsConsumed) : unitsConsumed;

    // Apply multiplier and base adjustment for safety margin
    const { cuMultiplier = CU_MULTIPLIER, cuBase = CU_BASE } = config;
    const finalUnits = Math.ceil(unitsConsumedNum * cuMultiplier + cuBase);

    return finalUnits;
  } catch (error) {
    console.warn("Failed to simulate transaction for CU calculation:", error);
    return null;
  }
}

export function getPdaFactory(
  programId: Address,
): (seeds: Seed[]) => Promise<PdaResp> {
  return async (seeds: Seed[]) => {
    return await getProgramDerivedAddress({
      programAddress: programId,
      seeds,
    });
  };
}

export async function getOrCreateAtaInstructions(
  rpc: RpcAny,
  payer: KeyPairSigner,
  mintPubkey: Address,
  ownerPubkey: Address,
  tokenProgram: Address,
): Promise<{
  ata: Address;
  ixs: Ix[];
}> {
  // calculate the ATA address
  const ata = await getAssociatedTokenAccountAddress(
    mintPubkey,
    ownerPubkey,
    tokenProgram,
  );

  // check if the account exists and is properly initialized
  try {
    await rpc.getAccountInfo(ata).send();

    // account exists and is properly initialized
    return {
      ata,
      ixs: [],
    };
  } catch (_) {
    // create the ATA creation instruction
    const ix = getCreateAssociatedTokenInstruction({
      payer,
      ata,
      owner: ownerPubkey,
      mint: mintPubkey,
      tokenProgram,
      systemProgram: SYSTEM_PROGRAM_ADDRESS,
    });

    return {
      ata,
      ixs: [ix],
    };
  }
}

export function getTokenProgramFactory(rpc: RpcAny) {
  return async (mint: Address): Promise<Address> => {
    // check if it's SOL (represented by default address)
    if (mint === SYSTEM_PROGRAM_ADDRESS) {
      throw new Error(`Mint ${mint.toString()} represents Sol`);
    }

    // it's a token, so get the mint account to determine which token program owns it
    const mintAccount = await rpc.getAccountInfo(mint).send();
    if (!mintAccount) {
      throw new Error(`Mint account ${mint.toString()} not found`);
    }

    // determine if it's Token Program or Token 2022
    const mintAccountOwner = mintAccount.value?.owner;
    let tokenProgram: Address;

    if (mintAccountOwner === TOKEN_PROGRAM_ADDRESS) {
      tokenProgram = TOKEN_PROGRAM_ADDRESS;
    } else if (mintAccountOwner === TOKEN_2022_PROGRAM_ADDRESS) {
      tokenProgram = TOKEN_2022_PROGRAM_ADDRESS;
    } else {
      throw new Error(`Unknown token program: ${mintAccountOwner}`);
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
  rustType: RustIntType,
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
        `Value ${value} is out of range for u8 (0-${maxValues.u8})`,
      );
    }
    return Buffer.from([value]);
  }

  // Handle u16, u32 with anchor.BN for consistency
  if (rustType === "u16" || rustType === "u32") {
    const maxValue = maxValues[rustType] as number;
    if (value < 0 || value > maxValue || !Number.isInteger(value)) {
      throw new Error(
        `Value ${value} is out of range for ${rustType} (0-${maxValue})`,
      );
    }

    return new BN(value).toArrayLike(Buffer, "le", byteSizes[rustType]);
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
          `Value ${value} must be a non-negative integer for ${rustType}`,
        );
      }
      bigintValue = BigInt(value);
    }

    const maxValue = maxValues[rustType] as bigint;
    if (bigintValue < BigInt(0) || bigintValue > maxValue) {
      throw new Error(
        `Value ${bigintValue} is out of range for ${rustType} (0-${maxValue})`,
      );
    }

    // For very large numbers, we need to handle them as bigint
    return new BN(bigintValue.toString()).toArrayLike(
      Buffer,
      "le",
      byteSizes[rustType],
    );
  }

  throw new Error(`Unsupported Rust type: ${rustType}`);
}
