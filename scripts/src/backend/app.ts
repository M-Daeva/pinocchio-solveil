import { fetchConfig } from "../common/schema/codama/accounts/config";
import {
  getInitInstruction,
  getUpdateConfigInstruction,
  InitInput,
  InitInstructionDataArgs,
  UpdateConfigInput,
  UpdateConfigInstructionDataArgs,
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
  Instruction,
  AccountLookupMeta,
  AccountMeta,
  KeyPairSigner,
  SendAndConfirmTransactionWithSignersFunction,
  SimulateTransactionFunction,
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
      .getRecentPrioritizationFees([payerAddress])
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

async function handleTx(
  rpcUrl: string,
  sendAndConfirmTransaction: SendAndConfirmTransactionWithSignersFunction,
  simulateTransaction: SimulateTransactionFunction,
  sender: KeyPairSigner,
  signerList: CryptoKeyPair[],
  instructions: Instruction<
    string,
    readonly (AccountLookupMeta<string, string> | AccountMeta<string>)[]
  >[],
  computeConfig: ComputeConfig = {},
  isDisplayed: boolean = false,
) {
  const rpc = createSolanaRpc(rpcUrl);

  // Get latest blockhash for transaction
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  // Create transaction for simulation
  const simulationTx = createTransaction({
    version: "legacy",
    feePayer: sender.address,
    latestBlockhash,
    instructions,
  });

  // Get optimal priority fee and compute units in parallel
  const [optimalPriorityFee, optimalComputeUnits] = await Promise.all([
    getOptimalPriorityFee(rpc, sender.address, computeConfig),
    getOptimalComputeUnits(simulateTransaction, simulationTx, computeConfig),
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

async function init(
  args: IRegistry.InitArgs,
  revenueMint: Address,
  computeConfig: ComputeConfig = {},
  isDisplayed: boolean = false,
) {
  // const rpc = createSolanaRpc(NETWORK_CONFIG.DEVNET);
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

  const ixs = [
    getInitInstruction({
      ...ixAccs,
      ...ixArgs,
    }),
  ];

  return handleTx(
    NETWORK_CONFIG.DEVNET,
    sendAndConfirmTransaction,
    simulateTransaction,
    sender,
    signerList,
    ixs,
    computeConfig,
    isDisplayed,
  );
}

async function updateConfig(
  args: IRegistry.UpdateConfigArgs,
  computeConfig: ComputeConfig = {},
  isDisplayed: boolean = false,
) {
  const { sendAndConfirmTransaction, simulateTransaction } = createSolanaClient(
    {
      urlOrMoniker: "devnet",
    },
  );

  const sender = await loadKeypairSignerFromFile(rootPath(PATH.OWNER_KEYPAIR));

  const registryPda = new RegistryPda(REGISTRY_CPI_PROGRAM_ADDRESS);

  const [[config], [adminRotationState]] = await Promise.all([
    registryPda.config(),
    registryPda.adminRotationState(),
  ]);

  const signerList: CryptoKeyPair[] = [sender.keyPair];

  const ixAccs: XOR<UpdateConfigInput, UpdateConfigInstructionDataArgs> = {
    sender,
    config,
    adminRotationState,
  };

  const ADMIN = 0;
  const IS_PAUSED = 1;
  const ROTATION_TIMEOUT = 2;
  const REGISTRATION_FEE_AMOUNT = 3;
  const DATA_SIZE_RANGE = 4;

  // TODO: make it type safe
  // TODO: write helper
  let flags = new BitField();
  let admin = sender.address; // placeholder
  let isPaused = new BitField();
  let rotationTimeout = new Uint32();
  let registrationFeeAmount = new Uint64();
  let dataSizeRange = { min: new Uint32(), max: new Uint32() };

  if (args.admin) {
    flags.setFlag(ADMIN, true);
    admin = args.admin;
  }

  if (args.is_paused) {
    flags.setFlag(IS_PAUSED, true);
    isPaused.setBit(args.is_paused);
  }

  if (args.rotation_timeout) {
    flags.setFlag(ROTATION_TIMEOUT, true);
    rotationTimeout.set(args.rotation_timeout);
  }

  if (args.registration_fee_amount) {
    flags.setFlag(REGISTRATION_FEE_AMOUNT, true);
    registrationFeeAmount.set(BigInt(args.registration_fee_amount));
  }

  if (args.data_size_range) {
    flags.setFlag(DATA_SIZE_RANGE, true);
    dataSizeRange.min.set(args.data_size_range.min);
    dataSizeRange.max.set(args.data_size_range.max);
  }

  const ixArgs: UpdateConfigInstructionDataArgs = {
    flags: flags.getRaw(),
    admin,
    isPaused: isPaused.getRaw(),
    rotationTimeout: rotationTimeout.toArray(),
    registrationFeeAmount: registrationFeeAmount.toArray(),
    dataSizeRange: {
      min: dataSizeRange.min.toArray(),
      max: dataSizeRange.max.toArray(),
    },
  };

  const ixs = [
    getUpdateConfigInstruction({
      ...ixAccs,
      ...ixArgs,
    }),
  ];

  return handleTx(
    NETWORK_CONFIG.DEVNET,
    sendAndConfirmTransaction,
    simulateTransaction,
    sender,
    signerList,
    ixs,
    computeConfig,
    isDisplayed,
  );
}

async function queryConfig(isDisplayed: boolean = false) {
  const rpc = createSolanaRpc(NETWORK_CONFIG.DEVNET);

  const registryPda = new RegistryPda(REGISTRY_CPI_PROGRAM_ADDRESS);
  const [config] = await registryPda.config();

  const { data } = await fetchConfig(rpc, config);

  interface Config {
    admin: Address;
    isPaused: boolean;
    rotationTimeout: number;
    registrationFee: {
      amount: BigInt;
      asset: Address;
    };
    dataSizeRange: { min: number; max: number };
  }

  // TODO: BigInt, BN, math.BigNumber

  const res: Config = {
    admin: data.admin,
    isPaused: new BitField(data.isPaused).getBit(),
    rotationTimeout: new Uint32(...data.rotationTimeout).get(),
    registrationFee: {
      amount: new Uint64(
        data.registrationFee.amount as unknown as Uint8Array,
      ).get(),
      asset: data.registrationFee.asset,
    },
    dataSizeRange: {
      min: new Uint32(...data.dataSizeRange.min).get(),
      max: new Uint32(...data.dataSizeRange.max).get(),
    },
  };

  return logAndReturn(res, isDisplayed);
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
  // await updateConfig({ rotation_timeout: 24 * 3_600 });
  // await queryConfig(true);
}

main();
