import { readKeypairSigner } from "./utils";
import { PATH, REVENUE_MINT } from "../common/config";
import { getClient, l, li, tokenProgramFactory } from "../common/utils";
import { RegistryHelpers } from "../common/account/index2";
import { ChainHelpers, WSOL_MINT } from "../common/account/chain";
import { address, generateKeyPairSigner } from "gill";

// const addr = getAddressEncoder();
const awsm = address("fPcP9vGoowPikgu7oTRCJKHUvSNn9N5WZhYshR4UXyo");
const recipient = address("4aPycKEbgz5tpFozhX3M22vdhPKumM4dpLwFXoFyR8WW");

async function main() {
  const client = getClient("DEVNET");
  const sender = await readKeypairSigner(PATH.OWNER_KEYPAIR);
  const h = new RegistryHelpers(client, sender);

  const tokenProgram = tokenProgramFactory(client.rpc);
  const c = new ChainHelpers(tokenProgram, client, sender);

  // const mintSigner = await generateKeyPairSigner();
  // await c.createMint(6, mintSigner);

  // (async () => {
  //   const [sol, wsol] = await Promise.all([
  //     c.getBalance(sender.address),
  //     c.getTokenBalance(WSOL_MINT, sender.address),
  //   ]);
  //   li({ sol, wsol });
  // })();

  // // await c.wrapSol(1);
  // await c.unwrapSol();

  // (async () => {
  //   const [sol, wsol] = await Promise.all([
  //     c.getBalance(sender.address),
  //     c.getTokenBalance(WSOL_MINT, sender.address),
  //   ]);
  //   li({ sol, wsol });
  // })();
  // return;

  // (async () => {
  //   const [from, to] = await Promise.all([
  //     c.getTokenBalance(awsm, sender.address),
  //     c.getTokenBalance(awsm, recipient),
  //   ]);
  //   li({ from, to });
  // })();

  // // await c.wrapSol(1);
  // await c.mintTokens(1, awsm, sender.address);

  // (async () => {
  //   const [from, to] = await Promise.all([
  //     c.getTokenBalance(awsm, sender.address),
  //     c.getTokenBalance(awsm, recipient),
  //   ]);
  //   li({ from, to });
  // })();
  // return;

  // await h.exec.init(
  //   {
  //     rotationTimeout: 48 * 3_600,
  //     accountRegistrationFee: {
  //       amount: 1_000n,
  //       asset: REVENUE_MINT.DEVNET,
  //     },
  //   },
  //   {},
  //   true,
  // );

  //
  // const str32 = new TString32("Hello World!").getRaw();
  // li({ str32 });
  // li({ str: new TString32(str32).get() });
  // enum Target {
  //   Spl,
  //   Proxy,
  //   Route,
  // }
  // const targetNumber = new TEnum(Target, Target.Proxy).getRaw();
  // l({ targetNumber });
  // l({ targetEnum: new TEnum(Target, targetNumber).get() });
  //

  await h.query.config(true);
  await h.query.userCounter(true);
  await h.query.adminRotationState(true);
  await h.query.revenue(true);
  // await h.exec.updateConfig(
  //   { rotationTimeout: 24 * 3_600, isPaused: true },
  //   {},
  //   true,
  // );
  // await h.query.config(true);
}

main();
