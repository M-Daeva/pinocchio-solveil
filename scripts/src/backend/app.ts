// import { connect } from "solana-kite";
import { fetchConfig, Config } from "../common/schema/codama/accounts";
import {
  getInitInstruction,
  getInitInstructionDataEncoder,
  InitInput,
  InitInstruction,
  InitInstructionDataArgs,
} from "../common/schema/codama/instructions";
import { REGISTRY_CPI_PROGRAM_ADDRESS } from "../common/schema/codama/programs/registryCpi";
import {
  SYSTEM_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  getAssociatedTokenAccountAddress,
} from "gill/programs";
import {
  createTransaction,
  Instruction,
  getProgramDerivedAddress,
  getAddressEncoder,
  Address,
  ProgramDerivedAddressBump,
  ReadonlyUint8Array,
  address,
  createSolanaClient,
  createSolanaRpc,
  compileTransaction,
  signTransaction,
} from "gill";
import { BitField, Uint32 } from "../common/interfaces/primitives";
import { loadKeypairSignerFromFile } from "gill/node";
import { rootPath } from "./utils";
import { NETWORK_CONFIG, PATH, REVENUE_MINT } from "../common/config";

import * as IRegistry from "../common/interfaces/registry";
import { l, li, logAndReturn } from "../common/utils";

// import * as IARegistry from "../interfaces/registry.anchor";

// const addr = getAddressEncoder();

// to get account interface from input and instruction data args interfaces
type XOR<T, U> = Omit<T, keyof U> & Omit<U, keyof T>;

type Seed = ReadonlyUint8Array | string;
type PdaResp = readonly [Address<string>, ProgramDerivedAddressBump];

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
  // params: TxParams = {},
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
    sender,
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
    amount: new Uint32(),
    asset: sender.address, // placeholder
  };
  let accountDataSizeRange = { min: new Uint32(), max: new Uint32() };

  if (args.rotationTimeout) {
    flags.setFlag(ROTATION_TIMEOUT, true);
    rotationTimeout.set(args.rotationTimeout);
  }

  if (args.accountRegistrationFee) {
    flags.setFlag(ACCOUNT_REGISTRATION_FEE, true);
    accountRegistrationFee.amount.set(args.accountRegistrationFee.amount);
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

  const ix = getInitInstruction({
    ...ixAccs,
    ...ixArgs,
  });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const txRaw = createTransaction({
    version: "legacy",
    feePayer: sender.address,
    latestBlockhash,
    instructions: [ix],
    computeUnitLimit: 0,
  });

  const res = await simulateTransaction(txRaw);
  li(res);
  const {
    value: { unitsConsumed },
  } = res;

  const tx = createTransaction({
    version: txRaw.version,
    feePayer: txRaw.feePayer as any,
    latestBlockhash: txRaw.lifetimeConstraint,
    instructions: txRaw.instructions as any,
    computeUnitLimit: unitsConsumed,
  });

  const signedTransaction = await signTransaction(
    signerList,
    compileTransaction(tx),
  );

  const signature = await sendAndConfirmTransaction(signedTransaction);

  // TODO: tx response instead of signature
  return logAndReturn(signature, isDisplayed);
}

async function main() {
  await init(
    {
      rotationTimeout: 0,
    },
    REVENUE_MINT.DEVNET,
    true,
  );
}

main();
