import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import {
  AccountMeta,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { DataRecord, TxParams } from "../interfaces";
import { decryptDeserialize, serializeEncrypt } from "./converters";
import { generateEncryptionKey, MessageSigningWallet } from "./encryption";
import {
  getHandleTx,
  getOrCreateAtaInstructions,
  getTimestamp,
  getTokenProgramFactory,
  li,
  logAndReturn,
  numberToRustBuffer,
  publicKeyFromString,
} from "../../common/utils";

import * as IRegistry from "../interfaces/registry";
import * as IDexAdapter from "../interfaces/dex-adapter";

import * as IARegistry from "../interfaces/registry.anchor";
import * as IADexAdapter from "../interfaces/dex-adapter.anchor";

import { Registry } from "../schema/types/registry";
import { DexAdapter } from "../schema/types/dex_adapter";
import { ClmmMock } from "../schema/types/clmm_mock";

export class RegistryHelpers {
  private provider: anchor.AnchorProvider;
  private program: anchor.Program<Registry>;
  private sender: PublicKey;

  private handleTx: (
    instructions: anchor.web3.TransactionInstruction[],
    params: TxParams,
    isDisplayed: boolean
  ) => Promise<anchor.web3.TransactionSignature>;

  private getTokenProgram: (
    mint: anchor.web3.PublicKey
  ) => Promise<anchor.web3.PublicKey>;

  constructor(
    provider: anchor.AnchorProvider,
    program: anchor.Program<Registry>
  ) {
    this.provider = provider;
    this.program = program;
    this.sender = provider.wallet.publicKey;
    this.handleTx = getHandleTx(provider);
    this.getTokenProgram = getTokenProgramFactory(provider);
  }

  async tryInit(
    args: IRegistry.InitArgs,
    revenueMint: PublicKey,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .init(...IARegistry.convertInitArgs(args))
      .accounts({
        tokenProgram: await this.getTokenProgram(revenueMint),
        revenueMint,
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryUpdateConfig(
    args: IRegistry.UpdateConfigArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .updateConfig(...IARegistry.convertUpdateConfigArgs(args))
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryConfirmAdminRotation(
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .confirmAdminRotation()
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryWithdrawRevenue(
    args: IRegistry.WithdrawRevenueArgs,
    revenueMint: PublicKey,
    recipient?: PublicKey,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .withdrawRevenue(...IARegistry.convertWithdrawRevenueArgs(args))
      .accounts({
        tokenProgram: await this.getTokenProgram(revenueMint),
        revenueMint,
        sender: this.sender,
        recipient: recipient || this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryCreateAccount(
    maxDataSize: number,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const { lastUserId } = await this.queryUserCounter();
    const expectedUserId = lastUserId + 1;
    const [userAccountPda] = this.getUserAccountPda(expectedUserId);
    const [userRotationStatePda] = this.getUserRotationStatePda(expectedUserId);

    const ix = await this.program.methods
      .createAccount(maxDataSize)
      .accounts({
        sender: this.sender,
        userAccount: userAccountPda,
        userRotationState: userRotationStatePda,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryCreateAndActivateAccount(
    maxDataSize: number,
    revenueMint: PublicKey,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const { lastUserId } = await this.queryUserCounter();
    const expectedUserId = lastUserId + 1;
    const [userAccountPda] = this.getUserAccountPda(expectedUserId);
    const [userRotationStatePda] = this.getUserRotationStatePda(expectedUserId);

    const createIx = await this.program.methods
      .createAccount(maxDataSize)
      .accounts({
        sender: this.sender,
        userAccount: userAccountPda,
        userRotationState: userRotationStatePda,
      })
      .instruction();

    const activateIx = await this.program.methods
      .activateAccount(this.sender)
      .accounts({
        tokenProgram: await this.getTokenProgram(revenueMint),
        sender: this.sender,
        revenueMint,
      })
      .instruction();

    return await this.handleTx([createIx, activateIx], params, isDisplayed);
  }

  // get estimated tx cost in SOL
  async simulateCreateAccount(
    maxDataSize: number,
    lamportsPerCu: number = 10_000,
    isDisplayed: boolean = false
  ) {
    const { lastUserId } = await this.queryUserCounter();
    const expectedUserId = lastUserId + 1;
    const [userAccountPda] = this.getUserAccountPda(expectedUserId);
    const [userRotationStatePda] = this.getUserRotationStatePda(expectedUserId);

    const res = await this.program.methods
      .createAccount(maxDataSize)
      .accounts({
        sender: this.sender,
        userAccount: userAccountPda,
        userRotationState: userRotationStatePda,
      })
      .simulate();

    const cuRegex = /consumed\s+(\d+)\s+of\s+(\d+)\s+compute units/i;
    let cu = 0;

    for (const line of res.raw) {
      const match = line.match(cuRegex);
      if (match) {
        cu = parseInt(match[1], 10);
        break;
      }
    }

    const txPrice = (cu * lamportsPerCu) / LAMPORTS_PER_SOL;
    const info = {
      cu,
      lamportsPerCu,
      txPrice,
    };

    return logAndReturn(info, isDisplayed);
  }

  async tryCloseAccount(
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .closeAccount()
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryReopenAccount(
    args: IRegistry.ReopenAccountArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .reopenAccount(...IARegistry.convertReopenAccountArgs(args))
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryActivateAccount(
    args: IRegistry.ActivateAccountArgs,
    revenueMint: PublicKey,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .activateAccount(args.user || this.sender)
      .accounts({
        tokenProgram: await this.getTokenProgram(revenueMint),
        sender: this.sender,
        revenueMint,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryWriteData(
    wallet: MessageSigningWallet,
    data: DataRecord[],
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const encKey = await generateEncryptionKey(wallet);
    const timestamp = getTimestamp();
    const { value } = serializeEncrypt(encKey, timestamp, data);

    const ix = await this.program.methods
      .writeData(value, new anchor.BN(timestamp))
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryRequestAccountRotation(
    args: IRegistry.RequestAccountRotationArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .requestAccountRotation(
        ...IARegistry.convertRequestAccountRotationArgs(args)
      )
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryConfirmAccountRotation(
    prevOwner: PublicKey,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const [userIdPrePda] = this.getUserIdPda(prevOwner);
    const [userIdPda] = this.getUserIdPda(this.sender);
    const { id: userIdValuePre } = await this.queryUserId(prevOwner);
    const [userRotationStatePda] = this.getUserRotationStatePda(userIdValuePre);

    const ix = await this.program.methods
      .confirmAccountRotation()
      .accountsPartial({
        sender: this.sender,
        userIdPre: userIdPrePda,
        userId: userIdPda,
        userRotationState: userRotationStatePda,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async queryConfig(isDisplayed: boolean = false) {
    const [pda] = this.getConfigPda();
    const res = await this.program.account.config.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryUserCounter(isDisplayed: boolean = false) {
    const [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_counter")],
      this.program.programId
    );
    const res = await this.program.account.userCounter.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryAdminRotationState(isDisplayed: boolean = false) {
    const [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin_rotation_state")],
      this.program.programId
    );
    const res = await this.program.account.rotationState.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryUserId(user: PublicKey, isDisplayed: boolean = false) {
    const [pda] = this.getUserIdPda(user);
    const res = await this.program.account.userId.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryUserAccountById(id: number) {
    const [pda] = this.getUserAccountPda(id);
    return await this.program.account.userAccount.fetch(pda);
  }

  async queryUserAccount(user: PublicKey, isDisplayed: boolean = false) {
    const { id } = await this.queryUserId(user);
    const res = await this.queryUserAccountById(id);

    return logAndReturn(res, isDisplayed);
  }

  async queryUserAccountList(batchSize: number = 100): Promise<
    {
      id: number;
      data: string;
      nonce: anchor.BN;
      maxSize: number;
    }[]
  > {
    let userAccountList: {
      id: number;
      data: string;
      nonce: anchor.BN;
      maxSize: number;
    }[] = [];

    const { lastUserId } = await this.queryUserCounter();

    // Create all PDAs first
    const userAccountPdas: { id: number; pda: PublicKey }[] = [];
    for (let i = 1; i <= lastUserId; i++) {
      const [pda] = this.getUserAccountPda(i);
      userAccountPdas.push({ id: i, pda });
    }

    // Batch fetch accounts
    for (let i = 0; i < userAccountPdas.length; i += batchSize) {
      const batch = userAccountPdas.slice(i, i + batchSize);
      const pdas = batch.map((item) => item.pda);

      try {
        const accountList =
          await this.program.account.userAccount.fetchMultiple(pdas);

        // Process the batch
        for (let j = 0; j < accountList.length; j++) {
          const account = accountList[j];

          // Account exists
          if (account) {
            userAccountList.push({
              id: batch[j].id,
              ...account,
            });
          }
        }
      } catch (error) {
        li(`Error fetching batch starting at index ${i}: ${error}`);

        // Fallback to individual fetches for this batch
        for (const { id, pda } of batch) {
          try {
            const account = await this.program.account.userAccount.fetch(pda);

            userAccountList.push({
              id,
              ...account,
            });
          } catch (err) {
            li(`Failed to fetch account ${id}: ${err}`);
          }
        }
      }
    }

    return userAccountList;
  }

  async readUserData(
    wallet: MessageSigningWallet,
    dataEncrypted: string,
    nonce: anchor.BN,
    isDisplayed: boolean = false
  ) {
    const encKey = await generateEncryptionKey(wallet);
    const res: DataRecord[] = decryptDeserialize(
      encKey,
      nonce.toString(),
      dataEncrypted
    );

    return logAndReturn(res, isDisplayed);
  }

  async queryUserRotationState(user: PublicKey, isDisplayed: boolean = false) {
    const { id } = await this.queryUserId(user);
    const [pda] = this.getUserRotationStatePda(id);
    const res = await this.program.account.rotationState.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryRevenue(isDisplayed: boolean = false) {
    const [configPda] = this.getConfigPda();
    const {
      registrationFee: { asset },
    } = await this.program.account.config.fetch(configPda);

    const ata = await spl.getAssociatedTokenAddress(
      asset,
      configPda,
      true,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const res = await ChainHelpers.getAtaTokenBalance(
      this.provider.connection,
      ata
    );

    return logAndReturn(res, isDisplayed);
  }

  getConfigPda() {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      this.program.programId
    );
  }

  getUserIdPda(user: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("user_id"), user.toBuffer()],
      this.program.programId
    );
  }

  getUserAccountPda(id: number) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), numberToRustBuffer(id, "u32")],
      this.program.programId
    );
  }

  getUserRotationStatePda(id: number) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("user_rotation_state"), numberToRustBuffer(id, "u32")],
      this.program.programId
    );
  }
}

export class DexAdapterHelpers {
  private provider: anchor.AnchorProvider;
  private program: anchor.Program<DexAdapter>;
  private registryProgramId: PublicKey;
  private clmmMockProgram: anchor.Program<ClmmMock>;
  private sender: PublicKey;

  private handleTx: (
    instructions: anchor.web3.TransactionInstruction[],
    params: TxParams,
    isDisplayed: boolean
  ) => Promise<anchor.web3.TransactionSignature>;

  private getTokenProgram: (
    mint: anchor.web3.PublicKey
  ) => Promise<anchor.web3.PublicKey>;

  constructor(
    provider: anchor.AnchorProvider,
    program: anchor.Program<DexAdapter>,
    registryProgramId: PublicKey,
    clmmMockProgram: anchor.Program<ClmmMock>
  ) {
    this.provider = provider;
    this.program = program;
    this.registryProgramId = registryProgramId;
    this.clmmMockProgram = clmmMockProgram;
    this.sender = provider.wallet.publicKey;
    this.handleTx = getHandleTx(provider);
    this.getTokenProgram = getTokenProgramFactory(provider);
  }

  async tryInit(
    args: IDexAdapter.InitArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .init(...IADexAdapter.convertInitArgs(args))
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryUpdateConfig(
    args: IDexAdapter.UpdateConfigArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .updateConfig(...IADexAdapter.convertUpdateConfigArgs(args))
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async tryConfirmAdminRotation(
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .confirmAdminRotation()
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async trySaveRoute(
    args: IDexAdapter.SaveRouteArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const ix = await this.program.methods
      .saveRoute(...IADexAdapter.convertSaveRouteArgs(args))
      .accounts({
        sender: this.sender,
      })
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async trySwap(
    args: IDexAdapter.SwapArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const [tokenIn, tokenOut, amountIn, amountOutMinimum] =
      IADexAdapter.convertSwapArgs(args);

    // PDA
    const [bump] = this.getBumpPda();
    const [config] = this.getConfigPda();
    const [route] = this.getRoutePda(tokenIn, tokenOut);

    // ATA
    const inputTokenSenderAta = getAssociatedTokenAddressSync(
      tokenIn,
      this.sender
    );
    const outputTokenSenderAta = getAssociatedTokenAddressSync(
      tokenOut,
      this.sender
    );

    // Build remaining accounts for the route
    const remainingAccounts = await this.buildRemainingAccountsForRoute(
      tokenIn,
      tokenOut
    );

    const accounts = {
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram2022: spl.TOKEN_2022_PROGRAM_ID,
      memoProgram: new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
      clmmMockProgram: this.clmmMockProgram.programId,
      sender: this.sender,
      bump,
      config,
      route,
      inputTokenMint: tokenIn,
      outputTokenMint: tokenOut,
      inputTokenSenderAta,
      outputTokenSenderAta,
    };

    const ix = await this.program.methods
      .swap(amountIn, amountOutMinimum)
      .accounts(accounts)
      .remainingAccounts(remainingAccounts)
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async trySwapAndActivate(
    args: IDexAdapter.SwapArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const [tokenIn, tokenOut, amountIn, amountOutMinimum] =
      IADexAdapter.convertSwapArgs(args);

    // PDA
    const [bump] = this.getBumpPda();
    const [config] = this.getConfigPda();
    const [route] = this.getRoutePda(tokenIn, tokenOut);
    const [registryConfig, registryBump] = this.getRegistryConfigPda();
    const [registryUserId] = this.getRegistryUserIdPda();

    // ATA
    const inputTokenSenderAta = getAssociatedTokenAddressSync(
      tokenIn,
      this.sender
    );
    const outputTokenSenderAta = getAssociatedTokenAddressSync(
      tokenOut,
      this.sender
    );
    const revenueAppAta = getAssociatedTokenAddressSync(
      tokenOut,
      registryConfig,
      true
    );

    // Build remaining accounts for the route
    const remainingAccounts = await this.buildRemainingAccountsForRoute(
      tokenIn,
      tokenOut
    );

    const accounts = {
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram2022: spl.TOKEN_2022_PROGRAM_ID,
      memoProgram: new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
      clmmMockProgram: this.clmmMockProgram.programId,
      sender: this.sender,
      bump,
      config,
      route,
      registryProgram: this.registryProgramId,
      registryBump,
      registryConfig,
      registryUserId,
      inputTokenMint: tokenIn,
      outputTokenMint: tokenOut,
      inputTokenSenderAta,
      outputTokenSenderAta,
      revenueAppAta,
    };

    const ix = await this.program.methods
      .swapAndActivate(amountIn, amountOutMinimum)
      .accounts(accounts)
      .remainingAccounts(remainingAccounts)
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  async trySwapAndUnwrapSol(
    args: IDexAdapter.SwapArgs,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const [tokenIn, tokenOut, amountIn, amountOutMinimum] =
      IADexAdapter.convertSwapArgs(args);

    // PDA
    const [bump] = this.getBumpPda();
    const [config] = this.getConfigPda();
    const [route] = this.getRoutePda(tokenIn, tokenOut);

    // ATA
    const inputTokenSenderAta = getAssociatedTokenAddressSync(
      tokenIn,
      this.sender
    );
    const outputTokenSenderAta = getAssociatedTokenAddressSync(
      tokenOut,
      this.sender
    );

    // Build remaining accounts for the route
    const remainingAccounts = await this.buildRemainingAccountsForRoute(
      tokenIn,
      tokenOut
    );

    const accounts = {
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram2022: spl.TOKEN_2022_PROGRAM_ID,
      memoProgram: new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
      clmmMockProgram: this.clmmMockProgram.programId,
      sender: this.sender,
      bump,
      config,
      route,
      inputTokenMint: tokenIn,
      outputTokenMint: tokenOut,
      inputTokenSenderAta,
      outputTokenSenderAta,
    };

    const ix = await this.program.methods
      .swapAndUnwrapWsol(amountIn, amountOutMinimum)
      .accounts(accounts)
      .remainingAccounts(remainingAccounts)
      .instruction();

    return await this.handleTx([ix], params, isDisplayed);
  }

  private async buildRemainingAccountsForRoute(
    mintIn: PublicKey,
    mintOut: PublicKey
  ): Promise<AccountMeta[]> {
    // Query route from PDA and build token sequence
    const { value: routeItems } = await this.queryRoute(mintIn, mintOut);
    const tokenSequence = [mintIn, ...routeItems.map((x) => x.tokenOut)];

    const remainingAccounts: AccountMeta[] = [];

    // Build accounts for each hop in the route
    for (let i = 0; i < routeItems.length; i++) {
      const tokenA = tokenSequence[i]; // input token for this hop
      const tokenB = tokenSequence[i + 1]; // output token for this hop
      const ammConfigIndex = routeItems[i].ammIndex;

      // Sort mints (pools are created with sorted tokens)
      const [token0Mint, token1Mint] = this.sortMints(tokenA, tokenB);

      // Get PDAs for CLMM mock
      const [ammConfig] = this.getClmmMockAmmConfigPda(ammConfigIndex);
      const [poolState] = this.getClmmMockPoolStatePda(
        ammConfig,
        token0Mint,
        token1Mint
      );
      const [observationState] = this.getClmmMockObservationStatePda(poolState);

      // Determine vaults based on token order
      let inputVault: PublicKey;
      let outputVault: PublicKey;
      let outputMintForAccounts: PublicKey;

      if (tokenA.equals(token0Mint)) {
        [inputVault] = this.getClmmMockTokenVault0Pda(poolState, token0Mint);
        [outputVault] = this.getClmmMockTokenVault1Pda(poolState, token1Mint);
        outputMintForAccounts = token1Mint;
      } else {
        [inputVault] = this.getClmmMockTokenVault1Pda(poolState, token1Mint);
        [outputVault] = this.getClmmMockTokenVault0Pda(poolState, token0Mint);
        outputMintForAccounts = token0Mint;
      }

      // Output token account (user's ATA)
      const outputTokenAccount = getAssociatedTokenAddressSync(
        tokenB,
        this.sender
      );

      // Add accounts in the exact order expected by the program
      remainingAccounts.push(
        { pubkey: ammConfig, isSigner: false, isWritable: false }, // amm_config (readonly)
        { pubkey: poolState, isSigner: false, isWritable: true }, // pool_state (writable)
        { pubkey: outputTokenAccount, isSigner: false, isWritable: true }, // output_token_account (writable)
        { pubkey: inputVault, isSigner: false, isWritable: true }, // input_vault (writable)
        { pubkey: outputVault, isSigner: false, isWritable: true }, // output_vault (writable)
        { pubkey: outputMintForAccounts, isSigner: false, isWritable: false }, // output_mint (readonly)
        { pubkey: observationState, isSigner: false, isWritable: true } // observation_state (writable)
      );
    }

    return remainingAccounts;
  }

  private areMintsSorted(mintA: PublicKey, mintB: PublicKey) {
    return mintA <= mintB;
  }

  sortMints(mintA: PublicKey, mintB: PublicKey) {
    return this.areMintsSorted(mintA, mintB) ? [mintA, mintB] : [mintB, mintA];
  }

  getRegistryConfigPda() {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      this.registryProgramId
    );
  }

  getRegistryUserIdPda() {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("user_id"), this.sender.toBuffer()],
      this.registryProgramId
    );
  }

  getClmmMockAmmConfigPda(ammConfigIndex: number) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("amm_config"), numberToRustBuffer(ammConfigIndex, "u16")],
      this.clmmMockProgram.programId
    );
  }

  getClmmMockPoolStatePda(
    ammConfig: PublicKey,
    token0Mint: PublicKey,
    token1Mint: PublicKey
  ) {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool"),
        ammConfig.toBuffer(),
        token0Mint.toBuffer(),
        token1Mint.toBuffer(),
      ],
      this.clmmMockProgram.programId
    );
  }

  getClmmMockObservationStatePda(poolState: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("observation"), poolState.toBuffer()],
      this.clmmMockProgram.programId
    );
  }

  getClmmMockTokenVault0Pda(poolState: PublicKey, token0Mint: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault"), poolState.toBuffer(), token0Mint.toBuffer()],
      this.clmmMockProgram.programId
    );
  }

  getClmmMockTokenVault1Pda(poolState: PublicKey, token1Mint: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault"), poolState.toBuffer(), token1Mint.toBuffer()],
      this.clmmMockProgram.programId
    );
  }

  getBumpPda() {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("bump")],
      this.program.programId
    );
  }

  getConfigPda() {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      this.program.programId
    );
  }

  getAdminRotationStatePda() {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("admin_rotation_state")],
      this.program.programId
    );
  }

  getRoutePda(mintFirst: PublicKey, mintLast: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("route"), mintFirst.toBuffer(), mintLast.toBuffer()],
      this.program.programId
    );
  }

  async queryBump(isDisplayed: boolean = false) {
    const [pda] = this.getBumpPda();
    const res = await this.program.account.daBump.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryConfig(isDisplayed: boolean = false) {
    const [pda] = this.getConfigPda();
    // anchor generates types for all programs, we need daConfig instead of config here
    const res = await this.program.account.daConfig.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryAdminRotationState(isDisplayed: boolean = false) {
    const [pda] = this.getAdminRotationStatePda();
    const res = await this.program.account.rotationState.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryRoute(
    mintFirst: PublicKey,
    mintLast: PublicKey,
    isDisplayed: boolean = false
  ) {
    const [pda] = this.getRoutePda(mintFirst, mintLast);
    const res = await this.program.account.route.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryAmmConfig(ammConfigIndex: number, isDisplayed: boolean = false) {
    const [pda] = this.getClmmMockAmmConfigPda(ammConfigIndex);
    const res = await this.clmmMockProgram.account.ammConfig.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryAmmConfigByAddr(pda: PublicKey, isDisplayed: boolean = false) {
    const res = await this.clmmMockProgram.account.ammConfig.fetch(pda);

    return logAndReturn(res, isDisplayed);
  }

  async queryAmmPoolState(
    ammConfigIndex: number,
    mintA: PublicKey,
    mintB: PublicKey,
    isDisplayed: boolean = false
  ) {
    const [ammConfigPda] = this.getClmmMockAmmConfigPda(ammConfigIndex);
    const [token0Mint, token1Mint] = this.sortMints(mintA, mintB);
    const [poolStatePda] = this.getClmmMockPoolStatePda(
      ammConfigPda,
      token0Mint,
      token1Mint
    );

    const res = await this.clmmMockProgram.account.poolState.fetch(
      poolStatePda
    );

    return logAndReturn(res, isDisplayed);
  }
}

export class ChainHelpers {
  private provider: anchor.AnchorProvider;
  private handleTx: (
    instructions: anchor.web3.TransactionInstruction[],
    params: TxParams,
    isDisplayed: boolean
  ) => Promise<anchor.web3.TransactionSignature>;

  constructor(provider: anchor.AnchorProvider) {
    this.provider = provider;
    this.handleTx = getHandleTx(provider);
  }

  async requestAirdrop(
    publicKey: anchor.web3.PublicKey | string,
    amount: number
  ): Promise<anchor.web3.TransactionSignature> {
    const signature = await this.provider.connection.requestAirdrop(
      publicKeyFromString(publicKey),
      amount * anchor.web3.LAMPORTS_PER_SOL
    );

    const { blockhash, lastValidBlockHeight } =
      await this.provider.connection.getLatestBlockhash();

    await this.provider.connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature,
    });

    return signature;
  }

  async createMint(
    mintKeypair: anchor.web3.Keypair,
    decimals: number,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    // https://solanacookbook.com/references/token.html#how-to-create-a-new-token
    const rent = await spl.getMinimumBalanceForRentExemptMint(
      this.provider.connection
    );

    const instructions: anchor.web3.TransactionInstruction[] = [
      // create mint account
      SystemProgram.createAccount({
        fromPubkey: this.provider.wallet.publicKey,
        newAccountPubkey: mintKeypair.publicKey,
        space: spl.MINT_SIZE,
        lamports: rent,
        programId: spl.TOKEN_PROGRAM_ID,
      }),
      // init mint account
      spl.createInitializeMintInstruction(
        mintKeypair.publicKey,
        decimals,
        this.provider.wallet.publicKey, // mint authority
        this.provider.wallet.publicKey // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
      ),
    ];

    // pass the mint keypair as a signer
    const updatedParams = {
      ...params,
      signers: [...(params.signers || []), mintKeypair],
    };

    return await this.handleTx(instructions, updatedParams, isDisplayed);
  }

  async getOrCreateAta(
    mintPubkey: PublicKey,
    ownerPubkey: PublicKey,
    allowOwnerOffCurve = false,
    params: TxParams = {},
    isDisplayed: boolean = false
  ) {
    const { ata, ixs } = await getOrCreateAtaInstructions(
      this.provider.connection,
      this.provider.wallet.publicKey,
      mintPubkey,
      ownerPubkey,
      allowOwnerOffCurve
    );

    if (ixs.length) {
      const sig = await this.handleTx(ixs, params, isDisplayed);
      li({ createAta: sig });
    }

    return logAndReturn(ata, isDisplayed);
  }

  async mintTokens(
    amount: number,
    mint: PublicKey | string,
    recipient: PublicKey | string,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const pkMint = publicKeyFromString(mint);
    const pkRecipient = publicKeyFromString(recipient);

    const { ata: ataRecipient, ixs } = await getOrCreateAtaInstructions(
      this.provider.connection,
      this.provider.wallet.publicKey,
      pkMint,
      pkRecipient,
      true
    );

    const { decimals } = await spl.getMint(this.provider.connection, pkMint);

    const instructions: anchor.web3.TransactionInstruction[] = [
      ...ixs,
      spl.createMintToCheckedInstruction(
        pkMint,
        ataRecipient,
        this.provider.wallet.publicKey,
        amount * 10 ** decimals,
        decimals
      ),
    ];

    return await this.handleTx(instructions, params, isDisplayed);
  }

  async transferTokens(
    amount: number,
    mint: PublicKey | string,
    to: PublicKey | string,
    params: TxParams = {},
    isDisplayed: boolean = false
  ) {
    const pkFrom = this.provider.wallet.publicKey;
    const pkTo = publicKeyFromString(to);
    const pkMint = publicKeyFromString(mint);

    const [infoFrom, infoTo] = await Promise.all(
      [pkFrom, pkTo].map((owner) =>
        getOrCreateAtaInstructions(
          this.provider.connection,
          this.provider.wallet.publicKey,
          pkMint,
          owner,
          true
        )
      )
    );

    const { decimals } = await spl.getMint(this.provider.connection, pkMint);

    const instructions: anchor.web3.TransactionInstruction[] = [
      ...infoFrom.ixs,
      ...infoTo.ixs,
      spl.createTransferCheckedInstruction(
        infoFrom.ata,
        pkMint,
        infoTo.ata,
        this.provider.wallet.publicKey,
        amount * 10 ** decimals,
        decimals
      ),
    ];

    return await this.handleTx(instructions, params, isDisplayed);
  }

  async getBalance(
    publicKey: PublicKey | string,
    isDisplayed: boolean = false
  ): Promise<number> {
    const balance = await this.provider.connection.getBalance(
      publicKeyFromString(publicKey)
    );

    return logAndReturn(balance / anchor.web3.LAMPORTS_PER_SOL, isDisplayed);
  }

  static async getAtaTokenBalance(
    connection: anchor.web3.Connection,
    ownerAta: PublicKey
  ): Promise<number> {
    let uiAmount: number | null = 0;

    try {
      ({
        value: { uiAmount },
      } = await connection.getTokenAccountBalance(ownerAta));
    } catch (_) {}

    return uiAmount || 0;
  }

  async getTokenBalance(
    mint: PublicKey | string,
    owner: PublicKey | string,
    isDisplayed: boolean = false
  ): Promise<number> {
    const pkMint = publicKeyFromString(mint);
    const pkOwner = publicKeyFromString(owner);

    const ata = await spl.getAssociatedTokenAddress(
      pkMint,
      pkOwner,
      true,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const uiAmount = await ChainHelpers.getAtaTokenBalance(
      this.provider.connection,
      ata
    );

    return logAndReturn(uiAmount, isDisplayed);
  }

  async getTx(signature: string, isDisplayed: boolean = false) {
    const tx = await this.provider.connection.getParsedTransaction(signature);

    return logAndReturn(tx, isDisplayed);
  }

  // https://www.quicknode.com/guides/solana-development/transactions/how-to-use-priority-fees
  // https://www.quicknode.com/docs/solana/qn_estimatePriorityFees
  // https://dashboard.quicknode.com/endpoints
  async getCuPrice(
    endpoint: string,
    programId: PublicKey | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const myHeaders = new Headers();
    myHeaders.append("Content-Type", "application/json");

    const raw = JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "qn_estimatePriorityFees",
      params: {
        last_n_blocks: 100,
        api_version: 2,
        ...(programId ? { account: programId } : {}),
      },
    });

    const requestOptions = {
      method: "POST",
      headers: myHeaders,
      body: raw,
      redirect: "follow",
    };

    const res = await fetch(endpoint, {
      method: requestOptions.method,
      headers: requestOptions.headers,
      body: requestOptions.body,
      redirect: "follow",
    }).then((response) => response.text());

    const lamportsPerCu =
      Number(JSON.parse(res)?.result?.per_compute_unit?.medium) || 0;

    return logAndReturn(lamportsPerCu, isDisplayed);
  }

  async wrapSol(
    amount: number,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const owner = this.provider.wallet.publicKey;
    const amountInLamports = amount * anchor.web3.LAMPORTS_PER_SOL;

    // Get or create ATA for WSOL (native mint)
    const { ata: wsolAta, ixs: createAtaIxs } =
      await getOrCreateAtaInstructions(
        this.provider.connection,
        owner,
        spl.NATIVE_MINT, // WSOL mint address
        owner,
        false
      );

    const instructions: anchor.web3.TransactionInstruction[] = [
      ...createAtaIxs,
      // Transfer SOL to the WSOL token account
      SystemProgram.transfer({
        fromPubkey: owner,
        toPubkey: wsolAta,
        lamports: amountInLamports,
      }),
      // Sync native instruction to convert SOL to WSOL tokens
      spl.createSyncNativeInstruction(wsolAta),
    ];

    return await this.handleTx(instructions, params, isDisplayed);
  }

  async unwrapSol(
    amount?: number,
    params: TxParams = {},
    isDisplayed: boolean = false
  ): Promise<anchor.web3.TransactionSignature> {
    const owner = this.provider.wallet.publicKey;

    const wsolAta = await spl.getAssociatedTokenAddress(
      spl.NATIVE_MINT,
      owner,
      false
    );

    // Get current WSOL balance if amount not specified
    let amountToUnwrap = amount;
    if (!amountToUnwrap) {
      const balance = await this.getTokenBalance(spl.NATIVE_MINT, owner, false);
      amountToUnwrap = balance;
    }

    if (amountToUnwrap <= 0) {
      throw new Error("No WSOL balance to unwrap");
    }

    const { decimals } = await spl.getMint(
      this.provider.connection,
      spl.NATIVE_MINT
    );
    const amountInTokens = amountToUnwrap * 10 ** decimals;

    const instructions: anchor.web3.TransactionInstruction[] = [
      // Close the WSOL token account to unwrap all, or transfer specific amount first
      ...(amount
        ? [
            spl.createTransferCheckedInstruction(
              wsolAta,
              spl.NATIVE_MINT,
              wsolAta, // Transfer to self to adjust balance
              owner,
              amountInTokens,
              decimals
            ),
          ]
        : []),
      spl.createCloseAccountInstruction(
        wsolAta,
        owner, // Destination for remaining SOL
        owner // Owner
      ),
    ];

    return await this.handleTx(instructions, params, isDisplayed);
  }
}
