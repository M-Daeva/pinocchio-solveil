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
import { generateEncryptionKey, MessageSigningWallet } from "./encryption";
import {
  getOrCreateAtaInstructions,
  getTimestamp,
  tokenProgramFactory,
  handleTxFactory,
  li,
  logAndReturn,
  numberToRustBuffer,
  pdaFactory,
  getClient,
  handleTx,
} from "../utils";

import * as IRegistry from "../interfaces/registry";

import {
  getInitInstruction,
  InitInput,
  InitInstructionDataArgs,
} from "../schema/codama/instructions/init";
import BN from "bn.js";
import {
  address,
  Address,
  KeyPairSigner,
  lamports,
  LAMPORTS_PER_SOL,
} from "gill";
import { TxResponse } from "../interfaces/tx";
import {
  ConfirmAccountRotationInput,
  ConfirmAccountRotationInstructionData,
  ConfirmAccountRotationInstructionDataArgs,
  ConfirmAdminRotationInput,
  ConfirmAdminRotationInstructionDataArgs,
  fetchConfig,
  fetchRotationState,
  fetchUserAccount,
  fetchUserCounter,
  fetchUserId,
  getConfirmAccountRotationInstruction,
  getConfirmAdminRotationInstruction,
  getUpdateConfigInstruction,
  getWithdrawRevenueInstruction,
  REGISTRY_CPI_PROGRAM_ADDRESS,
  UpdateConfigInput,
  UpdateConfigInstructionDataArgs,
  WithdrawRevenueInput,
  WithdrawRevenueInstructionDataArgs,
} from "../schema/codama";
import {
  IConfirmAccountRotationInstructionDataArgs,
  IConfirmAdminRotationInstructionDataArgs,
  IInitInstructionDataArgs,
  IUpdateConfigInstructionDataArgs,
  IWithdrawRevenueInstructionDataArgs,
} from "../schema/codegen/registry-cpi/instructions";
import { PATH } from "../config";
import {
  decConfig,
  decRotationState,
  decUserAccount,
  decUserCounter,
  decUserId,
  encConfirmAccountRotationInstructionDataArgs,
  encConfirmAdminRotationInstructionDataArgs,
  encUpdateConfigInstructionDataArgs,
  encWithdrawRevenueInstructionDataArgs,
} from "../schema/codegen/registry-cpi/codecs";
import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  getAssociatedTokenAccountAddress,
  SYSTEM_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  WithdrawNonceAccountInstructionDataArgs,
} from "gill/programs";
import {
  IConfig,
  IRotationState,
  IUserAccount,
  IUserCounter,
  IUserId,
} from "../schema/codegen/registry-cpi/accounts";

export class RegistryHelpers {
  pda: RegistryPda;

  client: ClientAny;
  sender: KeyPairSigner;

  programId: Address;

  query: RegistryQuery;
  exec: RegistryExec;

  constructor(client: ClientAny, sender: KeyPairSigner) {
    this.pda = new RegistryPda();
    this.client = client;
    this.sender = sender;
    this.programId = REGISTRY_CPI_PROGRAM_ADDRESS;

    const tokenProgram = tokenProgramFactory(client.rpc);
    this.query = new RegistryQuery(client.rpc, this.pda, tokenProgram);
    this.exec = new RegistryExec(this.pda, client, sender, tokenProgram);
  }
}

// TODO: pda w/o parameters can be stored
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

  async userId(user: Address): Promise<PdaResp> {
    return this.pda(["user_id", user]);
  }

  async userAccount(userId: number): Promise<PdaResp> {
    return this.pda(["user_account", numberToRustBuffer(userId, "u32")]);
  }

  async userRotationState(userId: number): Promise<PdaResp> {
    return this.pda(["user_rotation_state", numberToRustBuffer(userId, "u32")]);
  }
}

class RegistryQuery {
  private pda: RegistryPda;
  private rpc: RpcAny;
  private tokenProgram: (mint: Address) => Promise<Address>;

  constructor(
    rpc: RpcAny,
    pda: RegistryPda,
    tokenProgram: (mint: Address) => Promise<Address>,
  ) {
    this.pda = pda;
    this.rpc = rpc;
    this.tokenProgram = tokenProgram;
  }

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

  async readUserData(
    wallet: MessageSigningWallet,
    dataEncrypted: string,
    nonce: bigint,
    isDisplayed: boolean = false,
  ) {
    const encKey = await generateEncryptionKey(wallet);
    const res: DataRecord[] = decryptDeserialize(
      encKey,
      nonce.toString(),
      dataEncrypted,
    );
    return logAndReturn(res, isDisplayed);
  }

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

  private pda: RegistryPda;

  private client: ClientAny;
  private sender: KeyPairSigner;

  private tokenProgram: (mint: Address) => Promise<Address>;
  private handleTx: (
    signerList: CryptoKeyPair[],
    instructions: Ix[],
    computeConfig: ComputeConfig,
    isDisplayed: boolean,
  ) => Promise<TxResponse>;

  constructor(
    pda: RegistryPda,
    client: ClientAny,
    sender: KeyPairSigner,
    tokenProgram: (mint: Address) => Promise<Address>,
  ) {
    this.pda = pda;
    this.client = client;
    this.sender = sender;
    this.tokenProgram = tokenProgram;
    this.handleTx = handleTxFactory(client, sender);
  }

  async init(
    args: IInitInstructionDataArgs,
    revenueMint: Address,
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
      getUpdateConfigInstruction({
        ...ixAccs,
        ...encUpdateConfigInstructionDataArgs(args),
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
    args: IConfirmAdminRotationInstructionDataArgs,
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
        ...encConfirmAdminRotationInstructionDataArgs(args),
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

  // async tryCreateAccount(
  //   maxDataSize: number,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const { lastUserId } = await this.queryUserCounter();
  //   const expectedUserId = lastUserId + 1;
  //   const [userAccountPda] = this.getUserAccountPda(expectedUserId);
  //   const [userRotationStatePda] = this.getUserRotationStatePda(expectedUserId);

  //   const ix = await this.program.methods
  //     .createAccount(maxDataSize)
  //     .accounts({
  //       sender: this.sender,
  //       userAccount: userAccountPda,
  //       userRotationState: userRotationStatePda,
  //     })
  //     .instruction();

  //   return this.handleTx([ix], params, isDisplayed);
  // }

  // async tryCreateAndActivateAccount(
  //   maxDataSize: number,
  //   revenueMint: PublicKey,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const { lastUserId } = await this.queryUserCounter();
  //   const expectedUserId = lastUserId + 1;
  //   const [userAccountPda] = this.getUserAccountPda(expectedUserId);
  //   const [userRotationStatePda] = this.getUserRotationStatePda(expectedUserId);

  //   const createIx = await this.program.methods
  //     .createAccount(maxDataSize)
  //     .accounts({
  //       sender: this.sender,
  //       userAccount: userAccountPda,
  //       userRotationState: userRotationStatePda,
  //     })
  //     .instruction();

  //   const activateIx = await this.program.methods
  //     .activateAccount(this.sender)
  //     .accounts({
  //       tokenProgram: await this.getTokenProgram(revenueMint),
  //       sender: this.sender,
  //       revenueMint,
  //     })
  //     .instruction();

  //   return this.handleTx([createIx, activateIx], params, isDisplayed);
  // }

  // // get estimated tx cost in SOL
  // async simulateCreateAccount(
  //   maxDataSize: number,
  //   lamportsPerCu: number = 10_000,
  //   isDisplayed: boolean = false,
  // ) {
  //   const { lastUserId } = await this.queryUserCounter();
  //   const expectedUserId = lastUserId + 1;
  //   const [userAccountPda] = this.getUserAccountPda(expectedUserId);
  //   const [userRotationStatePda] = this.getUserRotationStatePda(expectedUserId);

  //   const res = await this.program.methods
  //     .createAccount(maxDataSize)
  //     .accounts({
  //       sender: this.sender,
  //       userAccount: userAccountPda,
  //       userRotationState: userRotationStatePda,
  //     })
  //     .simulate();

  //   const cuRegex = /consumed\s+(\d+)\s+of\s+(\d+)\s+compute units/i;
  //   let cu = 0;

  //   for (const line of res.raw) {
  //     const match = line.match(cuRegex);
  //     if (match) {
  //       cu = parseInt(match[1], 10);
  //       break;
  //     }
  //   }

  //   const txPrice = (cu * lamportsPerCu) / LAMPORTS_PER_SOL;
  //   const info = {
  //     cu,
  //     lamportsPerCu,
  //     txPrice,
  //   };

  //   return logAndReturn(info, isDisplayed);
  // }

  // async tryCloseAccount(
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const ix = await this.program.methods
  //     .closeAccount()
  //     .accounts({
  //       sender: this.sender,
  //     })
  //     .instruction();

  //   return this.handleTx([ix], params, isDisplayed);
  // }

  // async tryReopenAccount(
  //   args: IRegistry.ReopenAccountArgs,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const ix = await this.program.methods
  //     .reopenAccount(...IARegistry.convertReopenAccountArgs(args))
  //     .accounts({
  //       sender: this.sender,
  //     })
  //     .instruction();

  //   return this.handleTx([ix], params, isDisplayed);
  // }

  // async tryActivateAccount(
  //   args: IRegistry.ActivateAccountArgs,
  //   revenueMint: PublicKey,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const ix = await this.program.methods
  //     .activateAccount(args.user || this.sender)
  //     .accounts({
  //       tokenProgram: await this.getTokenProgram(revenueMint),
  //       sender: this.sender,
  //       revenueMint,
  //     })
  //     .instruction();

  //   return this.handleTx([ix], params, isDisplayed);
  // }

  // async tryWriteData(
  //   wallet: MessageSigningWallet,
  //   data: DataRecord[],
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const encKey = await generateEncryptionKey(wallet);
  //   const timestamp = getTimestamp();
  //   const { value } = serializeEncrypt(encKey, timestamp, data);

  //   const ix = await this.program.methods
  //     .writeData(value, new BN(timestamp))
  //     .accounts({
  //       sender: this.sender,
  //     })
  //     .instruction();

  //   return this.handleTx([ix], params, isDisplayed);
  // }

  // async tryRequestAccountRotation(
  //   args: IRegistry.RequestAccountRotationArgs,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const ix = await this.program.methods
  //     .requestAccountRotation(
  //       ...IARegistry.convertRequestAccountRotationArgs(args),
  //     )
  //     .accounts({
  //       sender: this.sender,
  //     })
  //     .instruction();

  //   return this.handleTx([ix], params, isDisplayed);
  // }

  // async tryConfirmAccountRotation(
  //   prevOwner: PublicKey,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const [userIdPrePda] = this.getUserIdPda(prevOwner);
  //   const [userIdPda] = this.getUserIdPda(this.sender);
  //   const { id: userIdValuePre } = await this.queryUserId(prevOwner);
  //   const [userRotationStatePda] = this.getUserRotationStatePda(userIdValuePre);

  //   const ix = await this.program.methods
  //     .confirmAccountRotation()
  //     .accountsPartial({
  //       sender: this.sender,
  //       userIdPre: userIdPrePda,
  //       userId: userIdPda,
  //       userRotationState: userRotationStatePda,
  //     })
  //     .instruction();

  //   return this.handleTx([ix], params, isDisplayed);
  // }
}

export class ChainHelpers {
  private client: ClientAny;
  private handleTx: (
    signerList: CryptoKeyPair[],
    instructions: Ix[],
    computeConfig: ComputeConfig,
    isDisplayed: boolean,
  ) => Promise<TxResponse>;

  constructor(client: ClientAny, sender: KeyPairSigner) {
    this.client = client;
    this.handleTx = handleTxFactory(client, sender);
  }

  async requestAirdrop(
    recipient: Address | string,
    amount: number,
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const rpc = this.client.rpc as RpcDev;

    let lamportsAmount = lamports(BigInt(amount) * BigInt(LAMPORTS_PER_SOL));
    const signature = await rpc
      .requestAirdrop(address(recipient), lamportsAmount)
      .send();

    const txResponse = await rpc.getTransaction(signature).send();

    return logAndReturn(txResponse as TxResponse, isDisplayed);
  }

  // async createMint(
  //   mintKeypair: anchor.web3.Keypair,
  //   decimals: number,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   // https://solanacookbook.com/references/token.html#how-to-create-a-new-token
  //   const rent = await spl.getMinimumBalanceForRentExemptMint(
  //     this.provider.connection,
  //   );

  //   const instructions: anchor.web3.TransactionInstruction[] = [
  //     // create mint account
  //     SystemProgram.createAccount({
  //       fromPubkey: this.provider.wallet.publicKey,
  //       newAccountPubkey: mintKeypair.publicKey,
  //       space: spl.MINT_SIZE,
  //       lamports: rent,
  //       programId: spl.TOKEN_PROGRAM_ID,
  //     }),
  //     // init mint account
  //     spl.createInitializeMintInstruction(
  //       mintKeypair.publicKey,
  //       decimals,
  //       this.provider.wallet.publicKey, // mint authority
  //       this.provider.wallet.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
  //     ),
  //   ];

  //   // pass the mint keypair as a signer
  //   const updatedParams = {
  //     ...params,
  //     signers: [...(params.signers || []), mintKeypair],
  //   };

  //   return this.handleTx(instructions, updatedParams, isDisplayed);
  // }

  // async getOrCreateAta(
  //   mintPubkey: PublicKey,
  //   ownerPubkey: PublicKey,
  //   allowOwnerOffCurve = false,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ) {
  //   const { ata, ixs } = await getOrCreateAtaInstructions(
  //     this.provider.connection,
  //     this.provider.wallet.publicKey,
  //     mintPubkey,
  //     ownerPubkey,
  //     allowOwnerOffCurve,
  //   );

  //   if (ixs.length) {
  //     const sig = await this.handleTx(ixs, params, isDisplayed);
  //     li({ createAta: sig });
  //   }

  //   return logAndReturn(ata, isDisplayed);
  // }

  // async mintTokens(
  //   amount: number,
  //   mint: PublicKey | string,
  //   recipient: PublicKey | string,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const pkMint = publicKeyFromString(mint);
  //   const pkRecipient = publicKeyFromString(recipient);

  //   const { ata: ataRecipient, ixs } = await getOrCreateAtaInstructions(
  //     this.provider.connection,
  //     this.provider.wallet.publicKey,
  //     pkMint,
  //     pkRecipient,
  //     true,
  //   );

  //   const { decimals } = await spl.getMint(this.provider.connection, pkMint);

  //   const instructions: anchor.web3.TransactionInstruction[] = [
  //     ...ixs,
  //     spl.createMintToCheckedInstruction(
  //       pkMint,
  //       ataRecipient,
  //       this.provider.wallet.publicKey,
  //       amount * 10 ** decimals,
  //       decimals,
  //     ),
  //   ];

  //   return this.handleTx(instructions, params, isDisplayed);
  // }

  // async transferTokens(
  //   amount: number,
  //   mint: PublicKey | string,
  //   to: PublicKey | string,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ) {
  //   const pkFrom = this.provider.wallet.publicKey;
  //   const pkTo = publicKeyFromString(to);
  //   const pkMint = publicKeyFromString(mint);

  //   const [infoFrom, infoTo] = await Promise.all(
  //     [pkFrom, pkTo].map((owner) =>
  //       getOrCreateAtaInstructions(
  //         this.provider.connection,
  //         this.provider.wallet.publicKey,
  //         pkMint,
  //         owner,
  //         true,
  //       ),
  //     ),
  //   );

  //   const { decimals } = await spl.getMint(this.provider.connection, pkMint);

  //   const instructions: anchor.web3.TransactionInstruction[] = [
  //     ...infoFrom.ixs,
  //     ...infoTo.ixs,
  //     spl.createTransferCheckedInstruction(
  //       infoFrom.ata,
  //       pkMint,
  //       infoTo.ata,
  //       this.provider.wallet.publicKey,
  //       amount * 10 ** decimals,
  //       decimals,
  //     ),
  //   ];

  //   return this.handleTx(instructions, params, isDisplayed);
  // }

  // async getBalance(
  //   publicKey: PublicKey | string,
  //   isDisplayed: boolean = false,
  // ): Promise<number> {
  //   const balance = await this.provider.connection.getBalance(
  //     publicKeyFromString(publicKey),
  //   );

  //   return logAndReturn(balance / anchor.web3.LAMPORTS_PER_SOL, isDisplayed);
  // }

  // static async getAtaTokenBalance(
  //   connection: anchor.web3.Connection,
  //   ownerAta: PublicKey,
  // ): Promise<number> {
  //   let uiAmount: number | null = 0;

  //   try {
  //     ({
  //       value: { uiAmount },
  //     } = await connection.getTokenAccountBalance(ownerAta));
  //   } catch (_) {}

  //   return uiAmount || 0;
  // }

  // TODO: test
  static async getAtaTokenBalance(
    rpc: RpcAny,
    ownerAta: Address,
  ): Promise<number> {
    let uiAmountString: string = "0";

    try {
      ({
        value: { uiAmountString },
      } = await rpc.getTokenAccountBalance(ownerAta).send());
    } catch (_) {}

    return Number(uiAmountString);
  }

  // async getTokenBalance(
  //   mint: PublicKey | string,
  //   owner: PublicKey | string,
  //   isDisplayed: boolean = false,
  // ): Promise<number> {
  //   const pkMint = publicKeyFromString(mint);
  //   const pkOwner = publicKeyFromString(owner);

  //   const ata = await spl.getAssociatedTokenAddress(
  //     pkMint,
  //     pkOwner,
  //     true,
  //     spl.TOKEN_PROGRAM_ID,
  //     spl.ASSOCIATED_TOKEN_PROGRAM_ID,
  //   );

  //   const uiAmount = await ChainHelpers.getAtaTokenBalance(
  //     this.provider.connection,
  //     ata,
  //   );

  //   return logAndReturn(uiAmount, isDisplayed);
  // }

  // async getTx(signature: string, isDisplayed: boolean = false) {
  //   const tx = await this.provider.connection.getParsedTransaction(signature);

  //   return logAndReturn(tx, isDisplayed);
  // }

  // // https://www.quicknode.com/guides/solana-development/transactions/how-to-use-priority-fees
  // // https://www.quicknode.com/docs/solana/qn_estimatePriorityFees
  // // https://dashboard.quicknode.com/endpoints
  // async getCuPrice(
  //   endpoint: string,
  //   programId: PublicKey | undefined = undefined,
  //   isDisplayed: boolean = false,
  // ) {
  //   const myHeaders = new Headers();
  //   myHeaders.append("Content-Type", "application/json");

  //   const raw = JSON.stringify({
  //     jsonrpc: "2.0",
  //     id: 1,
  //     method: "qn_estimatePriorityFees",
  //     params: {
  //       last_n_blocks: 100,
  //       api_version: 2,
  //       ...(programId ? { account: programId } : {}),
  //     },
  //   });

  //   const requestOptions = {
  //     method: "POST",
  //     headers: myHeaders,
  //     body: raw,
  //     redirect: "follow",
  //   };

  //   const res = await fetch(endpoint, {
  //     method: requestOptions.method,
  //     headers: requestOptions.headers,
  //     body: requestOptions.body,
  //     redirect: "follow",
  //   }).then((response) => response.text());

  //   const lamportsPerCu =
  //     Number(JSON.parse(res)?.result?.per_compute_unit?.medium) || 0;

  //   return logAndReturn(lamportsPerCu, isDisplayed);
  // }

  // async wrapSol(
  //   amount: number,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const owner = this.provider.wallet.publicKey;
  //   const amountInLamports = amount * anchor.web3.LAMPORTS_PER_SOL;

  //   // Get or create ATA for WSOL (native mint)
  //   const { ata: wsolAta, ixs: createAtaIxs } =
  //     await getOrCreateAtaInstructions(
  //       this.provider.connection,
  //       owner,
  //       spl.NATIVE_MINT, // WSOL mint address
  //       owner,
  //       false,
  //     );

  //   const instructions: anchor.web3.TransactionInstruction[] = [
  //     ...createAtaIxs,
  //     // Transfer SOL to the WSOL token account
  //     SystemProgram.transfer({
  //       fromPubkey: owner,
  //       toPubkey: wsolAta,
  //       lamports: amountInLamports,
  //     }),
  //     // Sync native instruction to convert SOL to WSOL tokens
  //     spl.createSyncNativeInstruction(wsolAta),
  //   ];

  //   return this.handleTx(instructions, params, isDisplayed);
  // }

  // async unwrapSol(
  //   amount?: number,
  //   params: TxParams = {},
  //   isDisplayed: boolean = false,
  // ): Promise<anchor.web3.TransactionSignature> {
  //   const owner = this.provider.wallet.publicKey;

  //   const wsolAta = await spl.getAssociatedTokenAddress(
  //     spl.NATIVE_MINT,
  //     owner,
  //     false,
  //   );

  //   // Get current WSOL balance if amount not specified
  //   let amountToUnwrap = amount;
  //   if (!amountToUnwrap) {
  //     const balance = await this.getTokenBalance(spl.NATIVE_MINT, owner, false);
  //     amountToUnwrap = balance;
  //   }

  //   if (amountToUnwrap <= 0) {
  //     throw new Error("No WSOL balance to unwrap");
  //   }

  //   const { decimals } = await spl.getMint(
  //     this.provider.connection,
  //     spl.NATIVE_MINT,
  //   );
  //   const amountInTokens = amountToUnwrap * 10 ** decimals;

  //   const instructions: anchor.web3.TransactionInstruction[] = [
  //     // Close the WSOL token account to unwrap all, or transfer specific amount first
  //     ...(amount
  //       ? [
  //           spl.createTransferCheckedInstruction(
  //             wsolAta,
  //             spl.NATIVE_MINT,
  //             wsolAta, // Transfer to self to adjust balance
  //             owner,
  //             amountInTokens,
  //             decimals,
  //           ),
  //         ]
  //       : []),
  //     spl.createCloseAccountInstruction(
  //       wsolAta,
  //       owner, // Destination for remaining SOL
  //       owner, // Owner
  //     ),
  //   ];

  //   return this.handleTx(instructions, params, isDisplayed);
  // }
}
