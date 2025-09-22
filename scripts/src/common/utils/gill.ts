// import { connect } from "solana-kite";
import { fetchConfig, Config } from "../schema/codama/accounts";
import {
  getInitInstruction,
  getInitInstructionDataEncoder,
  InitInput,
  InitInstruction,
  InitInstructionDataArgs,
} from "../schema/codama/instructions";
import { REGISTRY_CPI_PROGRAM_ADDRESS } from "../schema/codama/programs/registryCpi";
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
} from "gill";
import { BitField, Uint32 } from "../interfaces/primitives";

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

async function main() {
  const registryPda = new RegistryPda(REGISTRY_CPI_PROGRAM_ADDRESS);

  const [[bump], [config], [userCounter], [adminRotationState]] =
    await Promise.all([
      registryPda.bump(),
      registryPda.config(),
      registryPda.userCounter(),
      registryPda.adminRotationState(),
    ]);

  // TODO: signer
  const sender = address("");
  const revenueMint = address("");
  // TODO: get or create
  const revenueAppAta = await getAssociatedTokenAccountAddress(
    revenueMint,
    sender,
    TOKEN_PROGRAM_ADDRESS,
  );

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

  const ixArgs: InitInstructionDataArgs = {
    flags: new BitField(0b0000_0000).getRaw(),
    rotationTimeout: new Uint32(42).toArray(),
    accountRegistrationFee: { amount, asset },
    accountDataSizeRange: { min, max },
  };

  const ix = getInitInstruction({
    ...ixAccs,
    ...ixArgs,
  });
}

main();
