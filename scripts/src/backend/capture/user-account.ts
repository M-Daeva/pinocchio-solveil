import { writeFile } from "fs/promises";
import { COMMITMENT, PATH, UTILS } from "../../common/config";
import { getProgram, getProvider, getRpc, l } from "../../common/utils";
import { getWallet, readKeypair, rootPath } from "../utils";
import { RegistryHelpers } from "../../common/account";
import { getSnapshotPath } from "../utils";
import { Network } from "../../common/interfaces";

import { Registry } from "../../common/schema/types/registry";
import RegistryIdl from "../../common/schema/idl/registry.json";

const NETWORK: Network = "DEVNET";
const PAGINATION_QUERY_AMOUNT = 100;

async function main() {
  const { ENCODING } = UTILS;
  const ownerKeypair = await readKeypair(rootPath(PATH.OWNER_KEYPAIR));
  const provider = getProvider(
    getWallet(ownerKeypair),
    getRpc(NETWORK),
    COMMITMENT
  );

  const registryProgram = getProgram<Registry>(provider, RegistryIdl as any);
  const registry = new RegistryHelpers(provider, registryProgram);

  const userAccountList = await registry.queryUserAccountList(
    PAGINATION_QUERY_AMOUNT
  );

  try {
    // write files
    await writeFile(
      getSnapshotPath(NETWORK, "user-account.json"),
      JSON.stringify(userAccountList, null, 2),
      {
        encoding: ENCODING as any,
      }
    );
  } catch (error) {
    l(error);
  }
}

main();
