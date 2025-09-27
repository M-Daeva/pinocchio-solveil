import { fetchConfig } from "../common/schema/codama/accounts/config";
import { REGISTRY_CPI_PROGRAM_ADDRESS } from "../common/schema/codama/programs/registryCpi";
import { Address } from "gill";
import { TBitField, TUint32, TUint64 } from "../common/interfaces/primitives";
import { readKeypairSigner } from "./utils";
import { NETWORK_CONFIG, PATH, REVENUE_MINT } from "../common/config";
import {
  getClient,
  pdaFactory,
  getRpc,
  handleTx,
  l,
  li,
  logAndReturn,
} from "../common/utils";
import { ComputeConfig, PdaResp, Seed, XOR } from "../common/interfaces";
import {
  SYSTEM_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  getAssociatedTokenAccountAddress,
} from "gill/programs";
import {
  getInitInstruction,
  getUpdateConfigInstruction,
  InitInput,
  InitInstructionDataArgs,
  UpdateConfigInput,
  UpdateConfigInstructionDataArgs,
} from "../common/schema/codama/instructions";

import * as IRegistry from "../common/interfaces/registry";
import {
  decConfig,
  encUpdateConfigInstructionDataArgs,
  IConfig,
  IUpdateConfigInstructionDataArgs,
} from "../common/interfaces/codecs";

// const addr = getAddressEncoder();

class RegistryPda {
  private pda: (seeds: Seed[]) => Promise<PdaResp>;

  constructor() {
    this.pda = pdaFactory(REGISTRY_CPI_PROGRAM_ADDRESS);
  }

  async bump(): Promise<PdaResp> {
    return this.pda(["bump"]);
  }

  async config(): Promise<PdaResp> {
    return this.pda(["config"]);
  }

  async userCounter(): Promise<PdaResp> {
    return this.pda(["user_counter"]);
  }

  async adminRotationState(): Promise<PdaResp> {
    return this.pda(["admin_rotation_state"]);
  }
}

async function init(
  args: IRegistry.InitArgs,
  revenueMint: Address,
  computeConfig: ComputeConfig = {},
  isDisplayed: boolean = false,
) {
  const client = getClient("DEVNET");
  const sender = await readKeypairSigner(PATH.OWNER_KEYPAIR);

  const registryPda = new RegistryPda();

  const [[bump], [config], [userCounter], [adminRotationState]] =
    await Promise.all([
      registryPda.bump(),
      registryPda.config(),
      registryPda.userCounter(),
      registryPda.adminRotationState(),
    ]);

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

  let flags = new TBitField();
  let rotationTimeout = new TUint32();
  let accountRegistrationFee = {
    amount: new TUint64(),
    asset: sender.address, // placeholder
  };
  let accountDataSizeRange = { min: new TUint32(), max: new TUint32() };

  if (args.rotationTimeout) {
    flags.set(true, ROTATION_TIMEOUT);
    rotationTimeout.set(args.rotationTimeout);
  }

  if (args.accountRegistrationFee) {
    flags.set(true, ACCOUNT_REGISTRATION_FEE);
    accountRegistrationFee.amount.set(
      BigInt(args.accountRegistrationFee.amount),
    );
    accountRegistrationFee.asset = args.accountRegistrationFee.asset;
  }

  if (args.accountDataSizeRange) {
    flags.set(true, ACCOUNT_DATA_SIZE_RANGE);
    accountDataSizeRange.min.set(args.accountDataSizeRange.min);
    accountDataSizeRange.max.set(args.accountDataSizeRange.max);
  }

  const ixArgs: InitInstructionDataArgs = {
    flags: flags.getRaw(),
    rotationTimeout: rotationTimeout.getRaw(),
    accountRegistrationFee: {
      amount: accountRegistrationFee.amount.getRaw(),
      asset: accountRegistrationFee.asset,
    },
    accountDataSizeRange: {
      min: accountDataSizeRange.min.getRaw(),
      max: accountDataSizeRange.max.getRaw(),
    },
  };

  const ixs = [
    getInitInstruction({
      ...ixAccs,
      ...ixArgs,
    }),
  ];

  return handleTx(client, sender, signerList, ixs, computeConfig, isDisplayed);
}

async function updateConfig(
  args: IUpdateConfigInstructionDataArgs,
  computeConfig: ComputeConfig = {},
  isDisplayed: boolean = false,
) {
  const client = getClient("DEVNET");
  const sender = await readKeypairSigner(PATH.OWNER_KEYPAIR);

  const registryPda = new RegistryPda();

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

  const ixArgs = encUpdateConfigInstructionDataArgs(args);

  li({ ixArgs });
  return;

  // const ixs = [
  //   getUpdateConfigInstruction({
  //     ...ixAccs,
  //     ...ixArgs,
  //   }),
  // ];

  // return handleTx(client, sender, signerList, ixs, computeConfig, isDisplayed);
}

async function queryConfig(isDisplayed: boolean = false): Promise<IConfig> {
  const rpc = getRpc("DEVNET");
  const registryPda = new RegistryPda();
  const [config] = await registryPda.config();
  const { data } = await fetchConfig(rpc, config);

  li({ data });

  return logAndReturn(decConfig(data), isDisplayed);
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
  await updateConfig(
    { rotation_timeout: 24 * 3_600, is_paused: true },
    {},
    true,
  );
  // await queryConfig(true);
}

main();
