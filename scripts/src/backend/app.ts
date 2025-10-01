import { readKeypairSigner } from "./utils";
import { PATH, REVENUE_MINT } from "../common/config";
import { getClient, l, li } from "../common/utils";
import { RegistryHelpers } from "../common/account/index2";

// const addr = getAddressEncoder();

async function main() {
  const client = getClient("DEVNET");
  const sender = await readKeypairSigner(PATH.OWNER_KEYPAIR);
  const h = new RegistryHelpers(client, sender);

  // await h.exec.init(
  //   {
  //     rotationTimeout: 48 * 3_600,
  //   },
  //   REVENUE_MINT.DEVNET,
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
  // console.log({ targetNumber });
  // console.log({ targetEnum: new TEnum(Target, targetNumber).get() });
  //

  // TODO: find out why the program set default value of revenue mint on init
  // instead of fPcP9vGoowPikgu7oTRCJKHUvSNn9N5WZhYshR4UXyo

  await h.query.config(true);
  await h.query.userCounter(true); // TODO: check why default lastUserId: 3738929409
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
