import { readKeypairSigner } from "./utils";
import { PATH, REVENUE_MINT } from "../common/config";
import { getClient, l, li, tokenProgramFactory } from "../common/utils";
import { RegistryHelpers } from "../common/account/index2";
import { ChainHelpers } from "../common/account/chain";

// const addr = getAddressEncoder();

async function main() {
  const client = getClient("DEVNET");
  const sender = await readKeypairSigner(PATH.OWNER_KEYPAIR);
  const h = new RegistryHelpers(client, sender);

  const tokenProgram = tokenProgramFactory(client.rpc);
  const c = new ChainHelpers(tokenProgram, client, sender);

  await c.getBalance(sender.address, true);

  await c.wrapSol(1, { cuMultiplier: 1.3 });
  await c.getBalance(sender.address, true);
  return;

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
