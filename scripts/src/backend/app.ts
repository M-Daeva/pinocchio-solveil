import { fetchConfig } from "../common/schema/codama/accounts/config";
import { REGISTRY_CPI_PROGRAM_ADDRESS } from "../common/schema/codama/programs/registryCpi";
import { Address } from "gill";
import { BitField, Uint32, Uint64 } from "../common/interfaces/primitives";
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

// const addr = getAddressEncoder();

// TODO: add user pda
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

  return handleTx(client, sender, signerList, ixs, computeConfig, isDisplayed);
}

async function updateConfig(
  args: IRegistry.UpdateConfigArgs,
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

  return handleTx(client, sender, signerList, ixs, computeConfig, isDisplayed);
}

async function queryConfig(isDisplayed: boolean = false) {
  const rpc = getRpc("DEVNET");
  const registryPda = new RegistryPda();
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
  // await updateConfig({ rotation_timeout: 24 * 3_600 }, {}, true);
  // await queryConfig(true);
}

main();
