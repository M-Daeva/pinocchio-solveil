import {
  DataRecord,
  ComputeConfig,
  ClientAny,
  Ix,
  RpcDev,
  Seed,
  PdaResp,
  RpcAny,
  XOR,
} from "../interfaces";
import { decryptDeserialize, serializeEncrypt } from "./converters";
// import { generateEncryptionKey, MessageSigningWallet } from "./encryption";
import {
  getOrCreateAtaInstructions,
  getTimestamp,
  tokenProgramFactory,
  handleTxFactory,
  li,
  logAndReturn,
  numberToRustBuffer,
  pdaFactory,
} from "../utils";
import {
  address,
  Address,
  KeyPairSigner,
  lamports,
  LAMPORTS_PER_SOL,
} from "gill";
import { TxResponse } from "../interfaces/tx";
import {
  ActivateAccountInput,
  ActivateAccountInstructionDataArgs,
  CloseAccountInput,
  CloseAccountInstructionDataArgs,
  ConfirmAccountRotationInput,
  ConfirmAccountRotationInstructionDataArgs,
  ConfirmAdminRotationInput,
  ConfirmAdminRotationInstructionDataArgs,
  CreateAccountInput,
  CreateAccountInstructionDataArgs,
  fetchConfig,
  fetchRotationState,
  fetchUserAccount,
  fetchUserCounter,
  fetchUserId,
  getActivateAccountInstruction,
  getCloseAccountInstruction,
  getConfirmAccountRotationInstruction,
  getConfirmAdminRotationInstruction,
  getCreateAccountInstruction,
  getReopenAccountInstruction,
  getRequestAccountRotationInstruction,
  getUpdateConfigInstruction,
  getWithdrawRevenueInstruction,
  getWriteDataInstruction,
  REGISTRY_CPI_PROGRAM_ADDRESS,
  ReopenAccountInput,
  ReopenAccountInstructionDataArgs,
  RequestAccountRotationInput,
  RequestAccountRotationInstructionDataArgs,
  UpdateConfigInput,
  UpdateConfigInstructionDataArgs,
  WithdrawRevenueInput,
  WithdrawRevenueInstructionDataArgs,
  WriteDataInput,
  WriteDataInstructionDataArgs,
  getInitInstruction,
  InitInput,
  InitInstructionDataArgs,
} from "../schema/codama";
import {
  ICreateAccountInstructionDataArgs,
  IInitInstructionDataArgs,
  IReopenAccountInstructionDataArgs,
  IRequestAccountRotationInstructionDataArgs,
  IUpdateConfigInstructionDataArgs,
  IWithdrawRevenueInstructionDataArgs,
  IWriteDataInstructionDataArgs,
} from "../schema/codegen/registry-cpi/instructions";
import {
  decConfig,
  decRotationState,
  decUserAccount,
  decUserCounter,
  decUserId,
  encActivateAccountInstructionDataArgs,
  encCloseAccountInstructionDataArgs,
  encConfirmAccountRotationInstructionDataArgs,
  encConfirmAdminRotationInstructionDataArgs,
  encCreateAccountInstructionDataArgs,
  encInitInstructionDataArgs,
  encReopenAccountInstructionDataArgs,
  encRequestAccountRotationInstructionDataArgs,
  encUpdateConfigInstructionDataArgs,
  encWithdrawRevenueInstructionDataArgs,
  encWriteDataInstructionDataArgs,
} from "../schema/codegen/registry-cpi/codecs";
import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  getAssociatedTokenAccountAddress,
  SYSTEM_PROGRAM_ADDRESS,
} from "gill/programs";
import {
  IConfig,
  IRotationState,
  IUserAccount,
  IUserCounter,
  IUserId,
} from "../schema/codegen/registry-cpi/accounts";
import { ChainHelpers } from "./chain";

// TODO: use 1 file per contract

export class RegistryHelpers {
  programId: Address;
  pda: RegistryPda;
  query: RegistryQuery;
  exec: RegistryExec;

  constructor(client: ClientAny, sender: KeyPairSigner) {
    const tokenProgram = tokenProgramFactory(client.rpc);
    this.programId = REGISTRY_CPI_PROGRAM_ADDRESS;
    this.pda = new RegistryPda(this.programId);
    this.query = new RegistryQuery(tokenProgram, this.pda, client.rpc);
    this.exec = new RegistryExec(
      tokenProgram,
      this.pda,
      this.query,
      sender,
      client,
    );
  }
}

// TODO: pda w/o parameters can be stored
class RegistryPda {
  private pda: (seeds: Seed[]) => Promise<PdaResp>;

  constructor(programId: Address) {
    this.pda = pdaFactory(programId);
  }

  bump = async () => this.pda(["bump"]);
  config = async () => this.pda(["config"]);
  userCounter = async () => this.pda(["user_counter"]);
  adminRotationState = async () => this.pda(["admin_rotation_state"]);
  userId = async (user: Address) => this.pda(["user_id", user]);

  userAccount = async (userId: number) =>
    this.pda(["user_account", numberToRustBuffer(userId, "u32")]);

  userRotationState = async (userId: number) =>
    this.pda(["user_rotation_state", numberToRustBuffer(userId, "u32")]);
}

class RegistryQuery {
  constructor(
    private tokenProgram: (mint: Address) => Promise<Address>,
    private pda: RegistryPda,
    private rpc: RpcAny,
  ) {}

  async config(isDisplayed: boolean = false): Promise<IConfig> {
    const { rpc, pda } = this;
    const [config] = await pda.config();
    const { data } = await fetchConfig(rpc, config);
    return logAndReturn(decConfig(data), isDisplayed);
  }

  async userCounter(isDisplayed: boolean = false): Promise<IUserCounter> {
    const { rpc, pda } = this;
    const [userCounter] = await pda.config();
    const { data } = await fetchUserCounter(rpc, userCounter);
    return logAndReturn(decUserCounter(data), isDisplayed);
  }

  async adminRotationState(
    isDisplayed: boolean = false,
  ): Promise<IRotationState> {
    const { rpc, pda } = this;
    const [adminRotationState] = await pda.adminRotationState();
    const { data } = await fetchRotationState(rpc, adminRotationState);
    return logAndReturn(decRotationState(data), isDisplayed);
  }

  async userId(user: Address, isDisplayed: boolean = false): Promise<IUserId> {
    const { rpc, pda } = this;
    const [userId] = await pda.userId(user);
    const { data } = await fetchUserId(rpc, userId);
    return logAndReturn(decUserId(data), isDisplayed);
  }

  async userAccountById(
    id: number,
    isDisplayed: boolean = false,
  ): Promise<IUserAccount> {
    const { rpc, pda } = this;
    const [userAccount] = await pda.userAccount(id);
    const { data } = await fetchUserAccount(rpc, userAccount);
    return logAndReturn(decUserAccount(data), isDisplayed);
  }

  async userAccount(
    user: Address,
    isDisplayed: boolean = false,
  ): Promise<IUserAccount> {
    const { id } = await this.userId(user);
    return this.userAccountById(id, isDisplayed);
  }

  // TODO: test
  async userAccountList(batchSize: number = 100): Promise<
    {
      id: number;
      data: string;
      nonce: bigint;
      maxSize: number;
    }[]
  > {
    const { rpc, pda } = this;
    const { lastUserId } = await this.userCounter();
    let userAccountList: {
      id: number;
      data: string;
      nonce: bigint;
      maxSize: number;
    }[] = [];

    // Create all PDAs first
    const userAccountPdas: { id: number; pda: Address }[] = [];
    for (let i = 1; i <= lastUserId; i++) {
      const [userAccountPda] = await pda.userAccount(i);
      userAccountPdas.push({ id: i, pda: userAccountPda });
    }

    // Batch fetch accounts
    for (let i = 0; i < userAccountPdas.length; i += batchSize) {
      const batch = userAccountPdas.slice(i, i + batchSize);
      const pdaList = batch.map((item) => item.pda);

      try {
        const response = await rpc
          .getMultipleAccounts(pdaList, {
            encoding: "base64",
            commitment: "confirmed",
          })
          .send();

        const accountList = response.value;

        // Process the batch
        for (let j = 0; j < accountList.length; j++) {
          const accountInfo = accountList[j];

          // Account exists
          if (accountInfo && accountInfo.data) {
            // Decode the account data based on your UserAccount structure
            // You'll need to implement deserializeUserAccount to match your program's account layout
            const decodedAccount = this.deserializeUserAccount(
              accountInfo.data,
            );

            userAccountList.push({
              id: batch[j]?.id || 0,
              ...decodedAccount,
            });
          }
        }
      } catch (error) {
        li(`Error fetching batch starting at index ${i}: ${error}`);

        // Fallback to individual fetches for this batch
        for (const { id, pda } of batch) {
          try {
            const response = await rpc
              .getAccountInfo(pda, {
                encoding: "base64",
                commitment: "confirmed",
              })
              .send();

            if (response.value && response.value.data) {
              const decodedAccount = this.deserializeUserAccount(
                response.value.data,
              );
              userAccountList.push({
                id,
                ...decodedAccount,
              });
            }
          } catch (err) {
            li(`Failed to fetch account ${id}: ${err}`);
          }
        }
      }
    }

    return userAccountList;
  }

  // TODO: replace
  // Helper method to deserialize UserAccount data
  private deserializeUserAccount(data: [string, "base64"]): {
    data: string;
    nonce: bigint;
    maxSize: number;
  } {
    // Decode base64 data
    const buffer = Buffer.from(data[0], "base64");

    // Parse according to your UserAccount structure
    // This is a placeholder - adjust based on your actual account layout
    // Typically Anchor accounts have an 8-byte discriminator followed by the data

    // Example parsing (adjust offsets based on your struct):
    // Skip 8-byte discriminator
    let offset = 8;

    // Read data (assuming it's a string with 4-byte length prefix)
    const dataLength = buffer.readUInt32LE(offset);
    offset += 4;
    const dataStr = buffer.slice(offset, offset + dataLength).toString("utf8");
    offset += dataLength;

    // Read nonce (u8 in Rust = 1 byte)
    const nonce = BigInt(buffer.readUInt8(offset));
    offset += 1;

    // Read maxSize (u32 in Rust = 4 bytes)
    const maxSize = buffer.readUInt32LE(offset);

    return {
      data: dataStr,
      nonce,
      maxSize,
    };
  }

  // async readUserData(
  //   wallet: MessageSigningWallet,
  //   dataEncrypted: string,
  //   nonce: bigint,
  //   isDisplayed: boolean = false,
  // ) {
  //   const encKey = await generateEncryptionKey(wallet);
  //   const res: DataRecord[] = decryptDeserialize(
  //     encKey,
  //     nonce.toString(),
  //     dataEncrypted,
  //   );
  //   return logAndReturn(res, isDisplayed);
  // }

  async userRotationState(
    user: Address,
    isDisplayed: boolean = false,
  ): Promise<IRotationState> {
    const { rpc, pda } = this;
    const { id } = await this.userId(user);
    const [userRotationState] = await pda.userRotationState(id);
    const { data } = await fetchRotationState(rpc, userRotationState);
    return logAndReturn(decRotationState(data), isDisplayed);
  }

  async revenue(isDisplayed: boolean = false) {
    const { rpc, pda } = this;
    const [configPda] = await pda.config();
    const {
      registrationFee: { asset },
    } = await this.config();

    const tokenProgram = await this.tokenProgram(asset);
    const ata = await getAssociatedTokenAccountAddress(
      asset,
      configPda,
      tokenProgram,
    );

    const res = await ChainHelpers.getAtaTokenBalance(rpc, ata);
    return logAndReturn(res, isDisplayed);
  }
}

export class RegistryExec {
  private systemProgram = SYSTEM_PROGRAM_ADDRESS;
  private associatedTokenProgram = ASSOCIATED_TOKEN_PROGRAM_ADDRESS;

  private handleTx: (
    signerList: CryptoKeyPair[],
    instructions: Ix[],
    computeConfig: ComputeConfig,
    isDisplayed: boolean,
  ) => Promise<TxResponse>;

  constructor(
    private tokenProgram: (mint: Address) => Promise<Address>,
    private pda: RegistryPda,
    private query: RegistryQuery,
    private sender: KeyPairSigner,
    client: ClientAny,
  ) {
    this.handleTx = handleTxFactory(client, sender);
  }

  async init(
    args: IInitInstructionDataArgs,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda, systemProgram, associatedTokenProgram } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const [[bump], [config], [userCounter], [adminRotationState]] =
      await Promise.all([
        pda.bump(),
        pda.config(),
        pda.userCounter(),
        pda.adminRotationState(),
      ]);

    const revenueMint = args.accountRegistrationFee?.asset;
    if (!revenueMint) throw new Error("revenueMint isn't specified!");

    const tokenProgram = await this.tokenProgram(revenueMint);
    const revenueAppAta = await getAssociatedTokenAccountAddress(
      revenueMint,
      config,
      tokenProgram,
    );

    const ixAccs: XOR<InitInput, InitInstructionDataArgs> = {
      systemProgram,
      tokenProgram,
      associatedTokenProgram,
      sender,
      bump,
      config,
      userCounter,
      adminRotationState,
      revenueMint,
      revenueAppAta,
    };

    const ixs = [
      getInitInstruction({
        ...ixAccs,
        ...encInitInstructionDataArgs(args),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async updateConfig(
    args: IUpdateConfigInstructionDataArgs,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const [[config], [adminRotationState]] = await Promise.all([
      pda.config(),
      pda.adminRotationState(),
    ]);

    const ixAccs: XOR<UpdateConfigInput, UpdateConfigInstructionDataArgs> = {
      sender,
      config,
      adminRotationState,
    };

    const ixs = [
      getUpdateConfigInstruction({
        ...ixAccs,
        ...encUpdateConfigInstructionDataArgs(args),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async confirmAdminRotation(
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const [[config], [adminRotationState]] = await Promise.all([
      pda.config(),
      pda.adminRotationState(),
    ]);

    const ixAccs: XOR<
      ConfirmAdminRotationInput,
      ConfirmAdminRotationInstructionDataArgs
    > = {
      sender,
      config,
      adminRotationState,
    };

    const ixs = [
      getConfirmAdminRotationInstruction({
        ...ixAccs,
        ...encConfirmAdminRotationInstructionDataArgs({}),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async withdrawRevenue(
    args: IWithdrawRevenueInstructionDataArgs,
    revenueMint: Address,
    recipient?: Address,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda, systemProgram, associatedTokenProgram } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];
    recipient = recipient || sender.address;

    const [[bump], [config]] = await Promise.all([pda.bump(), pda.config()]);

    const tokenProgram = await this.tokenProgram(revenueMint);
    const [revenueRecipientAta, revenueAppAta] = await Promise.all([
      getAssociatedTokenAccountAddress(revenueMint, recipient, tokenProgram),
      getAssociatedTokenAccountAddress(revenueMint, config, tokenProgram),
    ]);

    const ixAccs: XOR<
      WithdrawRevenueInput,
      WithdrawRevenueInstructionDataArgs
    > = {
      systemProgram,
      tokenProgram,
      associatedTokenProgram,
      sender,
      recipient,
      bump,
      config,
      revenueMint,
      revenueRecipientAta,
      revenueAppAta,
    };

    const ixs = [
      getWithdrawRevenueInstruction({
        ...ixAccs,
        ...encWithdrawRevenueInstructionDataArgs(args),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async createAccount(
    args: ICreateAccountInstructionDataArgs,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda, systemProgram } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const { lastUserId } = await this.query.userCounter();
    const expectedUserId = lastUserId + 1;

    const [
      [bump],
      [config],
      [userCounter],
      [userId],
      [userAccount],
      [userRotationState],
    ] = await Promise.all([
      pda.bump(),
      pda.config(),
      pda.userCounter(),
      pda.userId(sender.address),
      pda.userAccount(expectedUserId),
      pda.userRotationState(expectedUserId),
    ]);

    const ixAccs: XOR<CreateAccountInput, CreateAccountInstructionDataArgs> = {
      systemProgram,
      sender,
      bump,
      config,
      userCounter,
      userId,
      userAccount,
      userRotationState,
    };

    const ixs = [
      getCreateAccountInstruction({
        ...ixAccs,
        ...encCreateAccountInstructionDataArgs(args),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async createAndActivateAccount(
    args: ICreateAccountInstructionDataArgs,
    revenueMint: Address,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda, systemProgram, associatedTokenProgram } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const { lastUserId } = await this.query.userCounter();
    const expectedUserId = lastUserId + 1;

    const [
      [bump],
      [config],
      [userCounter],
      [userId],
      [userAccount],
      [userRotationState],
    ] = await Promise.all([
      pda.bump(),
      pda.config(),
      pda.userCounter(),
      pda.userId(sender.address),
      pda.userAccount(expectedUserId),
      pda.userRotationState(expectedUserId),
    ]);

    const tokenProgram = await this.tokenProgram(revenueMint);
    const [revenueSenderAta, revenueAppAta] = await Promise.all([
      getAssociatedTokenAccountAddress(
        revenueMint,
        sender.address,
        tokenProgram,
      ),
      getAssociatedTokenAccountAddress(revenueMint, config, tokenProgram),
    ]);

    const createIx: XOR<CreateAccountInput, CreateAccountInstructionDataArgs> =
      {
        systemProgram,
        sender,
        bump,
        config,
        userCounter,
        userId,
        userAccount,
        userRotationState,
      };

    const activateIx: XOR<
      ActivateAccountInput,
      ActivateAccountInstructionDataArgs
    > = {
      systemProgram,
      tokenProgram,
      associatedTokenProgram,
      sender,
      bump,
      config,
      userId,
      revenueMint,
      revenueSenderAta,
      revenueAppAta,
    };

    const ixs = [
      getCreateAccountInstruction({
        ...createIx,
        ...encCreateAccountInstructionDataArgs(args),
      }),
      getActivateAccountInstruction({
        ...activateIx,
        ...encActivateAccountInstructionDataArgs({}),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async closeAccount(
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda, systemProgram } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const { id } = await this.query.userId(sender.address);

    const [[userId], [userAccount], [userRotationState]] = await Promise.all([
      pda.userId(sender.address),
      pda.userAccount(id),
      pda.userRotationState(id),
    ]);

    const ixAccs: XOR<CloseAccountInput, CloseAccountInstructionDataArgs> = {
      systemProgram,
      sender,
      userId,
      userAccount,
      userRotationState,
    };

    const ixs = [
      getCloseAccountInstruction({
        ...ixAccs,
        ...encCloseAccountInstructionDataArgs({}),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async reopenAccount(
    args: IReopenAccountInstructionDataArgs,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda, systemProgram } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const { id } = await this.query.userId(sender.address);

    const [[bump], [config], [userId], [userAccount], [userRotationState]] =
      await Promise.all([
        pda.bump(),
        pda.config(),
        pda.userId(sender.address),
        pda.userAccount(id),
        pda.userRotationState(id),
      ]);

    const ixAccs: XOR<ReopenAccountInput, ReopenAccountInstructionDataArgs> = {
      systemProgram,
      sender,
      bump,
      config,
      userId,
      userAccount,
      userRotationState,
    };

    const ixs = [
      getReopenAccountInstruction({
        ...ixAccs,
        ...encReopenAccountInstructionDataArgs(args),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async activateAccount(
    revenueMint: Address,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda, systemProgram, associatedTokenProgram } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const [[bump], [config], [userId]] = await Promise.all([
      pda.bump(),
      pda.config(),
      pda.userId(sender.address),
    ]);

    const tokenProgram = await this.tokenProgram(revenueMint);
    const [revenueSenderAta, revenueAppAta] = await Promise.all([
      getAssociatedTokenAccountAddress(
        revenueMint,
        sender.address,
        tokenProgram,
      ),
      getAssociatedTokenAccountAddress(revenueMint, config, tokenProgram),
    ]);

    const ixAccs: XOR<
      ActivateAccountInput,
      ActivateAccountInstructionDataArgs
    > = {
      systemProgram,
      tokenProgram,
      associatedTokenProgram,
      sender,
      bump,
      config,
      userId,
      revenueMint,
      revenueSenderAta,
      revenueAppAta,
    };

    const ixs = [
      getActivateAccountInstruction({
        ...ixAccs,
        ...encActivateAccountInstructionDataArgs({}),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  // async writeData(
  //   wallet: MessageSigningWallet,
  //   data: DataRecord[],
  //   computeConfig: ComputeConfig = {},
  //   isDisplayed: boolean = false,
  // ): Promise<TxResponse> {
  //   const { sender, pda } = this;
  //   const signerList: CryptoKeyPair[] = [sender.keyPair];

  //   const { id } = await this.query.userId(sender.address);
  //   const encKey = await generateEncryptionKey(wallet);
  //   const timestamp = getTimestamp();
  //   const { value } = serializeEncrypt(encKey, timestamp, data);
  //   const args: IWriteDataInstructionDataArgs = {
  //     data: value,
  //     nonce: BigInt(timestamp),
  //   };

  //   const [[userId], [userAccount]] = await Promise.all([
  //     pda.userId(sender.address),
  //     pda.userAccount(id),
  //   ]);

  //   const ixAccs: XOR<WriteDataInput, WriteDataInstructionDataArgs> = {
  //     sender,
  //     userId,
  //     userAccount,
  //   };

  //   const ixs = [
  //     getWriteDataInstruction({
  //       ...ixAccs,
  //       ...encWriteDataInstructionDataArgs(args),
  //     }),
  //   ];

  //   return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  // }

  async requestAccountRotation(
    args: IRequestAccountRotationInstructionDataArgs,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const { id } = await this.query.userId(sender.address);

    const [[bump], [config], [userId], [userRotationState]] = await Promise.all(
      [
        pda.bump(),
        pda.config(),
        pda.userId(sender.address),
        pda.userRotationState(id),
      ],
    );

    const ixAccs: XOR<
      RequestAccountRotationInput,
      RequestAccountRotationInstructionDataArgs
    > = {
      sender,
      bump,
      config,
      userId,
      userRotationState,
    };

    const ixs = [
      getRequestAccountRotationInstruction({
        ...ixAccs,
        ...encRequestAccountRotationInstructionDataArgs(args),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async confirmAccountRotation(
    prevOwner: Address,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, pda, systemProgram } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const { id: userIdValuePre } = await this.query.userId(prevOwner);

    const [[userIdPre], [userId], [userRotationState]] = await Promise.all([
      pda.userId(prevOwner),
      pda.userId(sender.address),
      pda.userRotationState(userIdValuePre),
    ]);

    const ixAccs: XOR<
      ConfirmAccountRotationInput,
      ConfirmAccountRotationInstructionDataArgs
    > = {
      systemProgram,
      sender,
      userIdPre,
      userId,
      userRotationState,
    };

    const ixs = [
      getConfirmAccountRotationInstruction({
        ...ixAccs,
        ...encConfirmAccountRotationInstructionDataArgs({}),
      }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }
}
