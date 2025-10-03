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
} from "../utils";
import {
  getInitInstruction,
  InitInput,
  InitInstructionDataArgs,
} from "../schema/codama/instructions/init";
import {
  address,
  Address,
  KeyPairSigner,
  lamports,
  LAMPORTS_PER_SOL,
} from "gill";
import { TxResponse } from "../interfaces/tx";
import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  getAssociatedTokenAccountAddress,
  SYSTEM_PROGRAM_ADDRESS,
} from "gill/programs";

export class ChainHelpers {
  private handleTx: (
    signerList: CryptoKeyPair[],
    instructions: Ix[],
    computeConfig: ComputeConfig,
    isDisplayed: boolean,
  ) => Promise<TxResponse>;

  constructor(
    private client: ClientAny,
    private sender: KeyPairSigner,
  ) {
    this.handleTx = handleTxFactory(client, sender);
  }

  async requestAirdrop(
    recipient: Address,
    amount: number,
    isDisplayed: boolean = false,
  ): Promise<TxResponse> {
    const rpc = this.client.rpc as RpcDev;

    let lamportsAmount = lamports(BigInt(amount) * BigInt(LAMPORTS_PER_SOL));
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
