import * as anchor from "@coral-xyz/anchor";
import { COMMITMENT, PATH, REVENUE_MINT } from "../common/config";
import { getProgram, getProvider, getRpc, li } from "../common/utils";
import { getWallet, parseNetwork, readKeypair, rootPath } from "./utils";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  ChainHelpers,
  RegistryHelpers,
  DexAdapterHelpers,
} from "../common/account";

import { Registry } from "../common/schema/types/registry";
import RegistryIdl from "../common/schema/idl/registry.json";

import { DexAdapter } from "../common/schema/types/dex_adapter";
import DexAdapterIdl from "../common/schema/idl/dex_adapter.json";

import { ClmmMock } from "../common/schema/types/clmm_mock";
import ClmmMockIdl from "../common/schema/idl/clmm_mock.json";

async function main() {
  const ownerKeypair = await readKeypair(rootPath(PATH.OWNER_KEYPAIR));
  const provider = getProvider(
    getWallet(ownerKeypair),
    getRpc("DEVNET"),
    COMMITMENT
  );

  const chain = new ChainHelpers(provider);
  const TX_PARAMS = {
    cpu: { k: 1, b: 150 },
  };

  const mintWsol = new PublicKey("So11111111111111111111111111111111111111112");
  const mintUsdc = new PublicKey("USDCoctVLVnvTXBEuP9s8hntucdJokbo17RwHuNXemT");

  const registryProgram = getProgram<Registry>(provider, RegistryIdl as any);
  const dexAdapterProgram = getProgram<DexAdapter>(
    provider,
    DexAdapterIdl as any
  );
  const clmmMockProgram = getProgram<ClmmMock>(provider, ClmmMockIdl as any);

  const registryAddress = registryProgram.programId;
  const dexAdapterAddress = dexAdapterProgram.programId;
  const dexAddress = clmmMockProgram.programId;

  li({
    registry: registryAddress.toString(),
    dexAdapter: dexAdapterAddress.toString(),
    dex: dexAddress.toString(),
  });

  const registry = new RegistryHelpers(provider, registryProgram);
  const dexAdapter = new DexAdapterHelpers(
    provider,
    dexAdapterProgram,
    registryAddress,
    clmmMockProgram
  );

  // await registry.tryInit(
  //   { accountRegistrationFee: { amount: 100_000, asset: REVENUE_MINT.DEVNET } },
  //   REVENUE_MINT.DEVNET,
  //   TX_PARAMS
  // );
  // await registry.queryConfig(true);

  const user = new PublicKey("4aPycKEbgz5tpFozhX3M22vdhPKumM4dpLwFXoFyR8WW");
  // const userId = await registry.queryUserId(user, true);
  // await registry.queryUserAccount(user, true);

  // await registry.tryCreateAccount(420, TX_PARAMS);
  await registry.queryUserAccount(ownerKeypair.publicKey, true);
  // await registry.tryRequestAccountRotation({ newOwner: user }, TX_PARAMS);
  await registry.queryUserRotationState(ownerKeypair.publicKey, true);

  // await dexAdapter.tryInit(
  //   {
  //     registry: registryAddress,
  //     dex: dexAddress,
  //   },
  //   TX_PARAMS
  // );

  // await dexAdapter.trySaveRoute(
  //   {
  //     mintFirst: mintWsol,
  //     mintLast: mintUsdc,
  //     route: [{ ammIndex: 0, tokenOut: mintUsdc }],
  //   },
  //   TX_PARAMS
  // );

  // await dexAdapter.queryConfig(true);
  // await dexAdapter.queryRoute(mintWsol, mintUsdc, true);

  // await chain.getTokenBalance(mintWsol, ownerKeypair.publicKey, true);
  // await chain.wrapSol(5, TX_PARAMS);
  // await chain.getTokenBalance(mintWsol, ownerKeypair.publicKey, true);

  // Check if pool exists before swapping
  // const AMM_CONFIG_INDEX = 0;

  // await dexAdapter.queryAmmPoolState(
  //   AMM_CONFIG_INDEX,
  //   mintUsdc,
  //   mintWsol,
  //   true
  // );

  // (async () => {
  //   const balanceWsol = await chain.getTokenBalance(
  //     mintWsol,
  //     ownerKeypair.publicKey
  //   );
  //   const balanceUsdc = await chain.getTokenBalance(
  //     mintUsdc,
  //     ownerKeypair.publicKey
  //   );

  //   li({
  //     balanceWsol,
  //     balanceUsdc,
  //   });
  // })();

  // await dexAdapter.trySwap(
  //   {
  //     amountIn: 5 * LAMPORTS_PER_SOL,
  //     amountOutMinimum: 1,
  //     tokenIn: mintWsol,
  //     tokenOut: mintUsdc,
  //   },
  //   TX_PARAMS
  // );

  // (async () => {
  //   const balanceWsol = await chain.getTokenBalance(
  //     mintWsol,
  //     ownerKeypair.publicKey
  //   );
  //   const balanceUsdc = await chain.getTokenBalance(
  //     mintUsdc,
  //     ownerKeypair.publicKey
  //   );

  //   li({
  //     balanceWsol,
  //     balanceUsdc,
  //   });
  // })();
}

main();
