// import { connect } from "solana-kite";

import { fetchConfig } from "../common/schema/codama/accounts/config";
import {
  getInitInstruction,
  InitInput,
  InitInstructionDataArgs,
} from "../common/schema/codama/instructions";
import { REGISTRY_CPI_PROGRAM_ADDRESS } from "../common/schema/codama/programs/registryCpi";
import {
  SYSTEM_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  getAssociatedTokenAccountAddress,
  getSetComputeUnitLimitInstruction,
  getSetComputeUnitPriceInstruction,
} from "gill/programs";
import {
  createTransaction,
  getProgramDerivedAddress,
  Address,
  ProgramDerivedAddressBump,
  ReadonlyUint8Array,
  createSolanaClient,
  createSolanaRpc,
  compileTransaction,
  signTransaction,
  LAMPORTS_PER_SOL,
} from "gill";
import { BitField, Uint32, Uint64 } from "../common/interfaces/primitives";
import { loadKeypairSignerFromFile } from "gill/node";
import { rootPath } from "./utils";
import { NETWORK_CONFIG, PATH, REVENUE_MINT } from "../common/config";

import * as IRegistry from "../common/interfaces/registry";
import { l, li, logAndReturn } from "../common/utils";
// const addr = getAddressEncoder();

// to get account interface from input and instruction data args interfaces
type XOR<T, U> = Omit<T, keyof U> & Omit<U, keyof T>;

type Seed = ReadonlyUint8Array | string;
type PdaResp = readonly [Address<string>, ProgramDerivedAddressBump];

// Configuration for compute units and priority fees
interface ComputeConfig {
  priorityFeeMultiplier?: number; // multiplier for base priority fee (default: 1.2)
  priorityFeeBase?: number; // base fee in microlamports to add (default: 0)
  cuMultiplier?: number; // multiplier for simulated CU usage (default: 1.2)
  cuBase?: number; // base CU to add (default: 0)
  maxPriorityFee?: number; // max priority fee cap in microlamports
  minPriorityFee?: number; // min priority fee floor in microlamports
}

/**
 * Get recent prioritization fees for calculating optimal priority fee
 */
async function getOptimalPriorityFee(
  rpc: any,
  payerAddress: Address,
  config: ComputeConfig = {},
): Promise<number> {
  const PRIORITY_FEE_MULTIPLIER = 1.0;
  const PRIORITY_FEE_BASE = 0;
  const MAX_PRIORITY_FEE = 0.01 * LAMPORTS_PER_SOL; // 0.01 SOL cap
  const MIN_PRIORITY_FEE = 0;

  try {
    // Get recent prioritization fees
    const { value: prioritizationFees } = await rpc
      .getRecentPrioritizationFees({
        lockedWritableAccounts: [payerAddress],
      })
      .send();

    let basePriorityFee = 0;
    if (prioritizationFees && prioritizationFees.length > 0) {
      // Calculate median or average (using average here for consistency with your legacy code)
      basePriorityFee = Math.ceil(
        prioritizationFees.reduce(
          (acc: number, cur: any) => acc + cur.prioritizationFee,
          0,
        ) / prioritizationFees.length,
      );
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

    // Apply multiplier and base adjustment for safety margin
    const { cuMultiplier = CU_MULTIPLIER, cuBase = CU_BASE } = config;
    const finalUnits = Math.ceil(unitsConsumed * cuMultiplier + cuBase);

    return finalUnits;
  } catch (error) {
    console.warn("Failed to simulate transaction for CU calculation:", error);
    return null;
  }
}

function getPdaFactory(
  programId: Address,
): (seeds: Seed[]) => Promise<PdaResp> {
  return async (seeds: Seed[]) => {
    return await getProgramDerivedAddress({
      programAddress: programId,
      seeds,
    });
  };
}

// TODO: add user pda
class RegistryPda {
  private factory: (seeds: Seed[]) => Promise<PdaResp>;

  constructor(programId: Address) {
    this.factory = getPdaFactory(programId);
  }

  async bump(): Promise<PdaResp> {
    return this.factory(["bump"]);
  }

  async config(): Promise<PdaResp> {
    return this.factory(["config"]);
  }

  async userCounter(): Promise<PdaResp> {
    return this.factory(["user_counter"]);
  }

  async adminRotationState(): Promise<PdaResp> {
    return this.factory(["admin_rotation_state"]);
  }
}

async function init(
  args: IRegistry.InitArgs,
  revenueMint: Address,
  computeConfig: ComputeConfig = {},
  isDisplayed: boolean = false,
) {
  const rpc = createSolanaRpc(NETWORK_CONFIG.DEVNET);
  const { sendAndConfirmTransaction, simulateTransaction } = createSolanaClient(
    {
      urlOrMoniker: "devnet",
    },
  );

  const sender = await loadKeypairSignerFromFile(rootPath(PATH.OWNER_KEYPAIR));

  const registryPda = new RegistryPda(REGISTRY_CPI_PROGRAM_ADDRESS);

  const [[bump], [config], [userCounter], [adminRotationState]] =
    await Promise.all([
      registryPda.bump(),
      registryPda.config(),
      registryPda.userCounter(),
      registryPda.adminRotationState(),
    ]);

  // TODO: get or create
  const revenueAppAta = await getAssociatedTokenAccountAddress(
    revenueMint,
    config,
    TOKEN_PROGRAM_ADDRESS,
  );

  const signerList: CryptoKeyPair[] = [sender.keyPair];

  const ixAccs: XOR<InitInput, InitInstructionDataArgs> = {
    systemProgram: SYSTEM_PROGRAM_ADDRESS,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    sender,
    bump,
    config,
    userCounter,
    adminRotationState,
    revenueMint,
    revenueAppAta,
  };

  const ROTATION_TIMEOUT = 0;
  const ACCOUNT_REGISTRATION_FEE = 1;
  const ACCOUNT_DATA_SIZE_RANGE = 2;

  let flags = new BitField();
  let rotationTimeout = new Uint32();
  let accountRegistrationFee = {
    amount: new Uint64(),
    asset: sender.address, // placeholder
  };
  let accountDataSizeRange = { min: new Uint32(), max: new Uint32() };

  if (args.rotationTimeout) {
    flags.setFlag(ROTATION_TIMEOUT, true);
    rotationTimeout.set(args.rotationTimeout);
  }

  if (args.accountRegistrationFee) {
    flags.setFlag(ACCOUNT_REGISTRATION_FEE, true);
    accountRegistrationFee.amount.set(
      BigInt(args.accountRegistrationFee.amount),
    );
    accountRegistrationFee.asset = args.accountRegistrationFee.asset;
  }

  if (args.accountDataSizeRange) {
    flags.setFlag(ACCOUNT_DATA_SIZE_RANGE, true);
    accountDataSizeRange.min.set(args.accountDataSizeRange.min);
    accountDataSizeRange.max.set(args.accountDataSizeRange.max);
  }

  const ixArgs: InitInstructionDataArgs = {
    flags: flags.getRaw(),
    rotationTimeout: rotationTimeout.toArray(),
    accountRegistrationFee: {
      amount: accountRegistrationFee.amount.toArray(),
      asset: accountRegistrationFee.asset,
    },
    accountDataSizeRange: {
      min: accountDataSizeRange.min.toArray(),
      max: accountDataSizeRange.max.toArray(),
    },
  };

  const mainIx = getInitInstruction({
    ...ixAccs,
    ...ixArgs,
  });

  // Get latest blockhash for transaction
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  // Create transaction for simulation
  const simulationTx = createTransaction({
    version: "legacy",
    feePayer: sender.address,
    latestBlockhash,
    instructions: [mainIx],
  });

  // Get optimal priority fee and compute units in parallel
  const [optimalPriorityFee, optimalComputeUnits] = await Promise.all([
    getOptimalPriorityFee(rpc, sender.address, computeConfig),
    getOptimalComputeUnits(simulateTransaction, simulationTx, computeConfig),
  ]);

  // Build final instructions array with compute budget instructions
  const finalInstructions = [];

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
  finalInstructions.push(mainIx);

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

  const signature = await sendAndConfirmTransaction(signedTransaction);

  // Log compute configuration for debugging
  if (isDisplayed) {
    l("Transaction Configuration:");
    l(`- Priority Fee: ${optimalPriorityFee} microlamports`);
    l(`- Compute Units: ${optimalComputeUnits || "default"}`);
    l(`- Signature: ${signature}`);
  }

  return logAndReturn(signature, isDisplayed);
}

async function queryConfig(isDisplayed: boolean = false) {
  const rpc = createSolanaRpc(NETWORK_CONFIG.DEVNET);

  const registryPda = new RegistryPda(REGISTRY_CPI_PROGRAM_ADDRESS);
  const [config] = await registryPda.config();

  const { data } = await fetchConfig(rpc, config);

  // TODO: convert data to human readable form

  return logAndReturn(data, isDisplayed);
}

async function main() {
  // await init(
  //   {
  //     rotationTimeout: 48 * 3_600,
  //   },
  //   REVENUE_MINT.DEVNET,
  //   {},
  //   true,
  // );

  await queryConfig(true);
}

main();
