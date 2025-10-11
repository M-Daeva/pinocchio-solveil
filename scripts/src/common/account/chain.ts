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
  getCreateAtaIx,
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
} from "gill/programs";

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

  // async transferTokens(
  //   amount: number,
  //   mint: Address,
  //   to: Address,
  //   computeConfig: ComputeConfig = {},
  //   isDisplayed: boolean = false,
  // ) {
  //   const from = this.sender.address;
  //   const signerList: CryptoKeyPair[] = [this.sender.keyPair];

  //   const tokenProgram = await this.tokenProgram(mint);
  //   const [infoFrom, infoTo] = await Promise.all(
  //     [from, to].map((owner) =>
  //       getOrCreateAtaInstructions(
  //         this.client.rpc,
  //         this.sender,
  //         mint,
  //         owner,
  //         tokenProgram,
  //       ),
  //     ),
  //   );

  //   if (!infoFrom || !infoTo) throw new Error("ATA aren't found!");

  //   const { value } = await this.client.rpc.getAccountInfo(mint).send();
  //   const data = value?.data;

  //   if (!data) throw new Error("");

  //   const { decimals } = spl.unpackMint(mint as any, data);

  //   const ixs: Ix[] = [
  //     ...infoFrom.ixs,
  //     ...infoTo.ixs,
  //     getTransferCheckedInstruction({
  //       source: infoFrom.ata,
  //       mint,
  //       destination: infoTo.ata,
  //       authority: this.sender,
  //       amount: amount * 10 ** decimals,
  //       decimals,
  //     }),
  //   ];

  //   return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  // }

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

    const wsolMint = address(spl.NATIVE_MINT.toString());
    const amountInLamports = amountInSol * LAMPORTS_PER_SOL;

    // Get or create ATA for WSOL
    const tokenProgram = await this.tokenProgram(wsolMint);
    const { ata: wsolAta, ixs: createAtaIxs } =
      await getOrCreateAtaInstructions(
        client.rpc,
        sender,
        wsolMint,
        sender.address,
        tokenProgram,
      );

    const ixs: Ix[] = [
      ...createAtaIxs,
      // // Transfer SOL to the WSOL token account
      // getTransferInstruction({
      //   authority: sender,
      //   source: sender.address,
      //   destination: wsolAta,
      //   amount: amountInLamports,
      // }),
      // // Sync native instruction to convert SOL to WSOL tokens
      // getSyncNativeInstruction({ account: wsolAta }),
    ];

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }

  // async wrapSol(
  //   amountInSol: number,
  //   computeConfig: ComputeConfig = {},
  //   isDisplayed: boolean = false,
  // ): Promise<TxResponse> {
  //   const { sender, client } = this;
  //   const signerList: CryptoKeyPair[] = [sender.keyPair];

  //   const wsolMint = address(spl.NATIVE_MINT.toString());
  //   const amountInLamports = amountInSol * LAMPORTS_PER_SOL;

  //   const tokenProgram = address("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

  //   // Calculate ATA address
  //   const wsolAta = await getAssociatedTokenAccountAddress(
  //     wsolMint,
  //     sender.address,
  //     tokenProgram,
  //   );

  //   // Check if ATA exists
  //   const ataAccountInfo = await client.rpc
  //     .getAccountInfo(wsolAta, {
  //       encoding: "base64",
  //     })
  //     .send();

  //   const ixs = [];

  //   // Only create if it doesn't exist
  //   if (!ataAccountInfo.value) {
  //     const createIx = getCreateAssociatedTokenInstruction({
  //       payer: sender,
  //       ata: wsolAta,
  //       owner: sender.address,
  //       mint: wsolMint,
  //       tokenProgram,
  //       systemProgram: SYSTEM_PROGRAM_ADDRESS,
  //     });

  //     // const createIx = getCreateAtaIx(
  //     //   sender,
  //     //   wsolMint,
  //     //   sender.address,
  //     //   tokenProgram,
  //     //   wsolAta,
  //     // );
  //     ixs.push(createIx);
  //   }

  //   // Transfer SOL to the WSOL token account
  //   ixs.push(
  //     getTransferInstruction({
  //       authority: sender,
  //       source: sender.address,
  //       destination: wsolAta,
  //       amount: amountInLamports,
  //     }),
  //   );

  //   // Sync native instruction
  //   ixs.push(getSyncNativeInstruction({ account: wsolAta }));

  //   return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  // }

  // async wrapSol(
  //   amountInSol: number,
  //   computeConfig: ComputeConfig = {},
  //   isDisplayed: boolean = false,
  // ): Promise<TxResponse> {
  //   const { sender, client } = this;
  //   const signerList: CryptoKeyPair[] = [sender.keyPair];

  //   const wsolMint = address(spl.NATIVE_MINT.toString());
  //   const amountInLamports = amountInSol * LAMPORTS_PER_SOL;

  //   const tokenProgram = address("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

  //   // Calculate ATA address
  //   const wsolAta = await getAssociatedTokenAccountAddress(
  //     wsolMint,
  //     sender.address,
  //     tokenProgram,
  //   );

  //   // Check if ATA exists
  //   const ataAccountInfo = await client.rpc
  //     .getAccountInfo(wsolAta, {
  //       encoding: "base64",
  //     })
  //     .send();

  //   const ixs = [];

  //   // Only create ATA if it doesn't exist
  //   if (!ataAccountInfo.value) {
  //     const createIx = spl.createAssociatedTokenAccountInstruction(
  //       sender.address as any, // payer
  //       wsolAta as any, // ATA address
  //       sender.address as any, // owner
  //       wsolMint as any, // mint
  //       tokenProgram as any, // token program
  //     );

  //     const newIx: Ix = {
  //       programAddress: address(createIx.programId.toString()),
  //       data: createIx.data,
  //       accounts: createIx.keys.map((x) => {
  //         let role: AccountRole;

  //         if (x.isSigner) {
  //           role = x.isWritable
  //             ? AccountRole.WRITABLE_SIGNER
  //             : AccountRole.READONLY_SIGNER;
  //         } else {
  //           role = x.isWritable ? AccountRole.WRITABLE : AccountRole.READONLY;
  //         }

  //         return {
  //           address: address(x.pubkey.toString()),
  //           role,
  //         };
  //       }),
  //     };

  //     // const createIx = getCreateAssociatedTokenInstruction({
  //     //   payer: sender,
  //     //   ata: wsolAta,
  //     //   owner: sender.address,
  //     //   mint: wsolMint,
  //     //   tokenProgram,
  //     //   systemProgram: SYSTEM_PROGRAM_ADDRESS,
  //     // });

  //     ixs.push(newIx);
  //   }

  //   // Transfer SOL to the WSOL token account
  //   ixs.push(
  //     // SystemProgram.transfer({
  //     //   fromPubkey: sender.address,
  //     //   toPubkey: wsolAta,
  //     //   lamports: amountInLamports,
  //     // }),

  //     getTransferInstruction({
  //       authority: sender,
  //       source: sender.address,
  //       destination: wsolAta,
  //       amount: amountInLamports,
  //     }),
  //   );

  //   // Sync native instruction
  //   ixs.push(getSyncNativeInstruction({ account: wsolAta }));

  //   return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  // }

  // async wrapSol(
  //   amountInSol: number,
  //   computeConfig: ComputeConfig = {},
  //   isDisplayed: boolean = false,
  // ): Promise<TxResponse> {
  //   const { sender, client } = this;
  //   const signerList: CryptoKeyPair[] = [sender.keyPair];

  //   const wsolMint = address(spl.NATIVE_MINT.toString());
  //   const amountInLamports = amountInSol * LAMPORTS_PER_SOL;

  //   const tokenProgram = address("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"); // SPL Token Program 2022
  //   const ataProgram = address("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"); // Associated Token Account Program
  //   const systemProgram = address("11111111111111111111111111111111"); // System Program

  //   // Calculate ATA address
  //   const wsolAta = await getAssociatedTokenAccountAddress(
  //     wsolMint,
  //     sender.address,
  //     tokenProgram,
  //   );

  //   // Check if ATA exists
  //   const ataAccountInfo = await client.rpc
  //     .getAccountInfo(wsolAta, {
  //       encoding: "base64",
  //     })
  //     .send();

  //   const ixs: Ix[] = [];

  //   // Only create ATA if it doesn't exist
  //   if (!ataAccountInfo.value) {
  //     const createIx = spl.createAssociatedTokenAccountInstruction(
  //       sender.address as any, // payer
  //       wsolAta as any, // ATA address
  //       sender.address as any, // owner
  //       wsolMint as any, // mint
  //       tokenProgram as any, // token program
  //       systemProgram as any, // system program
  //     );

  //     const newIx: Ix = {
  //       programAddress: ataProgram, // Explicitly set to ATA program
  //       data: createIx.data,
  //       accounts: [
  //         { address: sender.address, role: AccountRole.WRITABLE_SIGNER }, // payer
  //         { address: wsolAta, role: AccountRole.WRITABLE }, // ATA
  //         { address: sender.address, role: AccountRole.READONLY }, // owner
  //         { address: wsolMint, role: AccountRole.READONLY }, // mint
  //         { address: systemProgram, role: AccountRole.READONLY }, // system program
  //         { address: tokenProgram, role: AccountRole.READONLY }, // token program
  //       ],
  //     };

  //     ixs.push(newIx);
  //   }

  //   // Transfer SOL to the WSOL token account
  //   // ixs.push({
  //   //   programAddress: systemProgram,
  //   //   data: Buffer.from([
  //   //     0x02,
  //   //     ...Buffer.alloc(8).writeBigUInt64LE(BigInt(amountInLamports)),
  //   //   ]), // SystemProgram.transfer
  //   //   accounts: [
  //   //     { address: sender.address, role: AccountRole.WRITABLE_SIGNER }, // from
  //   //     { address: wsolAta, role: AccountRole.WRITABLE }, // to
  //   //   ],
  //   // });

  //   ixs.push(
  //     getTransferInstruction({
  //       authority: sender,
  //       source: sender.address,
  //       destination: wsolAta,
  //       amount: amountInLamports,
  //     }),
  //   );

  //   // Sync native instruction
  //   ixs.push({
  //     programAddress: tokenProgram,
  //     data: Buffer.from([0x11]), // SyncNative instruction (ID 17)
  //     accounts: [
  //       { address: wsolAta, role: AccountRole.WRITABLE }, // account
  //     ],
  //   });

  //   return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  // }

  async unwrapSol(
    amountInSol?: number,
    computeConfig: ComputeConfig = {},
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const { sender } = this;
    const signerList: CryptoKeyPair[] = [sender.keyPair];

    const wsolMint = address(spl.NATIVE_MINT.toString());
    const tokenProgram = await this.tokenProgram(wsolMint);
    const wsolAta = await getAssociatedTokenAccountAddress(
      wsolMint,
      sender.address,
      tokenProgram,
    );

    let ixs: Ix[] = [];

    if (amountInSol) {
      // Unwrap specific amount: create temp account, transfer, close temp
      const amountInLamports = amountInSol * LAMPORTS_PER_SOL;

      // Create temporary WSOL account
      const tempAccount = await generateKeyPairSigner();
      signerList.push(tempAccount.keyPair);

      ixs = [
        // Create temporary token account
        getCreateAccountInstruction({
          payer: sender,
          newAccount: tempAccount,
          space: spl.ACCOUNT_SIZE,
          lamports: getMinimumBalanceForRentExemption(spl.ACCOUNT_SIZE),
          programAddress: tokenProgram,
        }),
        // Initialize it as a token account
        getInitializeAccountInstruction({
          account: tempAccount.address,
          mint: wsolMint,
          owner: sender.address,
        }),
        // Transfer specific amount to temp account
        getTransferInstruction({
          authority: sender,
          source: wsolAta,
          destination: tempAccount.address,
          amount: amountInLamports,
        }),
        // Close temp account to unwrap
        getCloseAccountInstruction({
          account: tempAccount.address,
          destination: sender.address,
          owner: sender,
        }),
      ];
    } else {
      // Unwrap all: just close the main account
      ixs = [
        getCloseAccountInstruction({
          account: wsolAta,
          destination: sender.address,
          owner: sender,
        }),
      ];
    }

    return this.handleTx(signerList, ixs, computeConfig, isDisplayed);
  }
}
