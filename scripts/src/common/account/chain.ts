import { TxResponse } from "../interfaces/tx";
import * as spl from "@solana/spl-token";
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
import {
  getOrCreateAtaInstructions,
  getTimestamp,
  tokenProgramFactory,
  handleTxFactory,
  li,
  logAndReturn,
  numberToRustBuffer,
  pdaFactory,
  numberFrom,
  l,
  getAccInfo,
} from "../utils";
import {
  getInitInstruction,
  InitInput,
  InitInstructionDataArgs,
} from "../schema/codama/instructions/init";
import {
  address,
  Address,
  getMinimumBalanceForRentExemption,
  getPublicKeyFromAddress,
  KeyPairSigner,
  lamports,
  LAMPORTS_PER_SOL,
  Signature,
  generateKeyPair,
  generateKeyPairSigner,
  AccountRole,
} from "gill";
import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  getAssociatedTokenAccountAddress,
  SYSTEM_PROGRAM_ADDRESS,
  getTransferCheckedInstruction,
  getSyncNativeInstruction,
  getTransferInstruction,
  getCloseAccountInstruction,
  getCreateAccountInstruction,
  getInitializeAccountInstruction,
  getCreateAssociatedTokenInstruction,
  TOKEN_PROGRAM_ADDRESS,
  getTransferSolInstruction,
} from "gill/programs";

export const WSOL_MINT = address("So11111111111111111111111111111111111111112");

export class ChainHelpers {
  private handleTx: (
    signerList: CryptoKeyPair[],
    instructions: Ix[],
    computeConfig: ComputeConfig,
    isDisplayed: boolean,
  ) => Promise<TxResponse>;

  constructor(
    private tokenProgram: (mint: Address) => Promise<Address>,
    private client: ClientAny,
    private sender: KeyPairSigner,
  ) {
    this.handleTx = handleTxFactory(client, sender);
  }

  async requestAirdrop(
    recipient: Address,
    amountInSol: number,
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const rpc = this.client.rpc as RpcDev;

    let lamportsAmount = lamports(
      BigInt(amountInSol) * BigInt(LAMPORTS_PER_SOL),
    );
    const signature = await rpc
      .requestAirdrop(recipient, lamportsAmount)
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

  async getOrCreateAta(
    mintPubkey: Address,
    ownerPubkey: Address,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ) {
    const signerList: CryptoKeyPair[] = [this.sender.keyPair];

    const tokenProgram = await this.tokenProgram(mintPubkey);
    const { ata, ixs } = await getOrCreateAtaInstructions(
      this.client.rpc,
      this.sender,
      mintPubkey,
      ownerPubkey,
      tokenProgram,
    );

    if (ixs.length) {
      await this.handleTx(signerList, ixs, computeConfig, isDisplayed);
    }

    return logAndReturn(ata, isDisplayed);
  }

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

  async transferTokens(
    amount: number,
    mint: Address,
    to: Address,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ) {
    const { sender, client } = this;
    const from = sender.address;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const tokenProgram = await this.tokenProgram(mint);
    const [infoFrom, infoTo] = await Promise.all(
      [from, to].map((owner) =>
        getOrCreateAtaInstructions(
          client.rpc,
          sender,
          mint,
          owner,
          tokenProgram,
        ),
      ),
    );

    if (!infoFrom || !infoTo) throw new Error("ATA aren't found!");

    const { value } = await getAccInfo(client.rpc, mint);
    const data = value?.data;
    if (!data) throw new Error("No mint data found");

    // Decode base64 data to buffer
    const buffer = Buffer.from(data[0], "base64");
    // Read decimals from byte offset 44
    const decimals = buffer.readUInt8(44);
    const amountInBaseUnits = amount * 10 ** decimals;

    const ixs: Ix[] = [
      ...infoFrom.ixs,
      ...infoTo.ixs,
      getTransferInstruction(
        {
          authority: sender,
          source: infoFrom.ata,
          destination: infoTo.ata,
          amount: amountInBaseUnits,
        },
        { programAddress: tokenProgram },
      ),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async getBalance(
    publicKey: Address,
    isDisplayed: boolean = false,
  ): Promise<number> {
    const { value } = await this.client.rpc.getBalance(publicKey).send();
    const res = numberFrom(value.toString())
      .div(LAMPORTS_PER_SOL)
      .toDecimalPlaces(9)
      .toNumber();

    return logAndReturn(res, isDisplayed);
  }

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

  async getTokenBalance(
    mint: Address,
    owner: Address,
    isDisplayed: boolean = false,
  ): Promise<number> {
    const tokenProgram = await this.tokenProgram(mint);
    const ata = await getAssociatedTokenAccountAddress(
      mint,
      owner,
      tokenProgram,
    );

    const uiAmount = await ChainHelpers.getAtaTokenBalance(
      this.client.rpc,
      ata,
    );

    return logAndReturn(uiAmount, isDisplayed);
  }

  async getTx(signature: Signature, isDisplayed: boolean = false) {
    const tx = await this.client.rpc.getTransaction(signature).send();

    return logAndReturn(tx, isDisplayed);
  }

  async wrapSol(
    amountInSol: number,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, client } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const amountInLamports = amountInSol * LAMPORTS_PER_SOL;

    // Get or create ATA for WSOL
    const tokenProgram = await this.tokenProgram(WSOL_MINT);
    const { ata: wsolAta, ixs: createAtaIxs } =
      await getOrCreateAtaInstructions(
        client.rpc,
        sender,
        WSOL_MINT,
        sender.address,
        tokenProgram,
      );

    const ixs: Ix[] = [
      ...createAtaIxs,
      // Transfer SOL to the WSOL token account
      getTransferSolInstruction({
        source: sender,
        destination: wsolAta,
        amount: amountInLamports,
      }),
      // Sync native instruction to convert SOL to WSOL tokens
      getSyncNativeInstruction(
        { account: wsolAta },
        { programAddress: tokenProgram }, // Explicitly use SPL Token, not Token-2022
      ),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  async unwrapSol(
    amountInSol?: number,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender, client } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const tokenProgram = await this.tokenProgram(WSOL_MINT);
    const wsolAta = await getAssociatedTokenAccountAddress(
      WSOL_MINT,
      sender.address,
      tokenProgram,
    );

    let ixs: Ix[] = [];

    if (amountInSol) {
      // Unwrap specific amount: transfer to temp account, then close it
      const amountInLamports = amountInSol * LAMPORTS_PER_SOL;

      // Create temporary token account
      const tempAccount = await generateKeyPairSigner();
      signerList.push(tempAccount.keyPair);

      const space = 165; // Token account size
      const rentExemption = await client.rpc
        .getMinimumBalanceForRentExemption(BigInt(space))
        .send();

      ixs = [
        // Create account
        getCreateAccountInstruction({
          payer: sender,
          newAccount: tempAccount,
          space,
          lamports: rentExemption,
          programAddress: tokenProgram,
        }),
        // Initialize token account
        getInitializeAccountInstruction(
          {
            account: tempAccount.address,
            mint: WSOL_MINT,
            owner: sender.address,
          },
          { programAddress: tokenProgram },
        ),
        // Transfer specific amount to temp account
        getTransferInstruction(
          {
            authority: sender,
            source: wsolAta,
            destination: tempAccount.address,
            amount: amountInLamports,
          },
          { programAddress: tokenProgram },
        ),
        // Close temp account to unwrap
        getCloseAccountInstruction(
          {
            account: tempAccount.address,
            destination: sender.address,
            owner: sender,
          },
          { programAddress: tokenProgram },
        ),
      ];
    } else {
      // Unwrap all: just close the main ATA
      ixs = [
        getCloseAccountInstruction(
          {
            account: wsolAta,
            destination: sender.address,
            owner: sender,
          },
          { programAddress: tokenProgram },
        ),
      ];
    }

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }
}
