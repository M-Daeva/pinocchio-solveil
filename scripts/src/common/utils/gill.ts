import { createTransaction, Instruction } from "gill";
import { fetchConfig, Config } from "../schema/codama/accounts";
import { getInitInstruction } from "../schema/codama/instructions";

async function main() {
  const ix: Instruction = {
    programAddress,
    accounts,
    data,
  };

  const transaction = createTransaction({
    version,
    feePayer,
    instructions,
    // the compute budget values are HIGHLY recommend to be set in order to maximize your transaction landing rate
    // computeUnitLimit: number,
    // computeUnitPrice: number,
  });
}

main();
