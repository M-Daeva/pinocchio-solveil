import { writeFile } from "fs/promises";
import { PATH, UTILS } from "../../common/config";
import { getClient, l } from "../../common/utils";
import { RegistryHelpers } from "../../common/account/registry";
import { getSnapshotPath, readKeypairSigner } from "../utils";
import { Network } from "../../common/interfaces";

const NETWORK: Network = "DEVNET";
const PAGINATION_QUERY_AMOUNT = 100;

async function main() {
  const { ENCODING } = UTILS;
  const client = getClient(NETWORK);
  const sender = await readKeypairSigner(PATH.OWNER_KEYPAIR);
  const registry = new RegistryHelpers(client, sender);

  const userAccountList = await registry.query.userAccountList(
    PAGINATION_QUERY_AMOUNT,
  );

  try {
    // write files
    await writeFile(
      getSnapshotPath(NETWORK, "user-account.json"),
      JSON.stringify(userAccountList, null, 2),
      {
        encoding: ENCODING as any,
      },
    );
  } catch (error) {
    l(error);
  }
}

main();
