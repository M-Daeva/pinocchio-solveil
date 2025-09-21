import { loadKeypairSignerFromFile } from "gill/dist/node";
import { readFileSync } from "fs";
import path from "path";
import {
  createSolanaRpc,
  createSolanaClient,
  createTransaction,
  signTransactionMessageWithSigners,
  getSignatureFromTransaction,
  lamports,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
  DevnetUrl,
  LAMPORTS_PER_SOL,
} from "gill";
import { rootPath } from "../utils";
import { PATH } from "../../common/config";

// Configuration
const RPC_URL = "https://api.devnet.solana.com"; // Change to mainnet for production
const PROGRAM_SO_PATH = "./target/deploy/your_program.so"; // Path to your compiled program

async function deployProgram() {
  try {
    console.log("🚀 Starting Solana program deployment with Gill...");

    // Create RPC connection
    const rpc = createSolanaRpc(RPC_URL);
    // const client = createSolanaClient({ rpc });
    const client = createSolanaClient({ urlOrMoniker: "devnet" });

    // TODO: client.simulateTransaction instead of getSimulationComputeUnits

    // Load the deployer keypair
    console.log("📝 Loading keypair...");
    const deployerSigner = await loadKeypairSignerFromFile(
      rootPath(PATH.OWNER_KEYPAIR),
    );
    console.log(`Deployer address: ${deployerSigner.address}`);

    // Check balance
    const balance = await rpc.getBalance(deployerSigner.address).send();
    console.log(
      `Deployer balance: ${balance.value / BigInt(LAMPORTS_PER_SOL)} SOL`,
    );

    // if (balance.value < lamports(1000000n)) {
    //   throw new Error(
    //     "Insufficient balance. Need at least 0.001 SOL for deployment.",
    //   );
    // }

    // // Read the program binary
    // console.log("📖 Reading program binary...");
    // const programBuffer = readFileSync(path.resolve(PROGRAM_SO_PATH));
    // console.log(`Program size: ${programBuffer.length} bytes`);

    // // Get recent blockhash
    // console.log("🔗 Getting recent blockhash...");
    // const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    // // Create program deployment transaction
    // console.log("🔨 Creating deployment transaction...");

    // // Calculate the minimum balance needed for the program account
    // const programAccountSpace = programBuffer.length;
    // const { value: minBalance } = await rpc
    //   .getMinimumBalanceForRentExemption(BigInt(programAccountSpace))
    //   .send();

    // // Create deployment instruction
    // const deployInstruction = createBpfLoaderUpgradeableDeployInstruction({
    //   payerAccount: deployerSigner.address,
    //   programDataAccount: deployerSigner.address, // This will be generated
    //   programAccount: deployerSigner.address, // This will be generated
    //   bufferAccount: deployerSigner.address, // This will be generated
    //   rentAccount: "11111111111111111111111111111111", // Rent sysvar
    //   clockAccount: "SysvarC1ock11111111111111111111111111111111", // Clock sysvar
    //   systemProgramAccount: "11111111111111111111111111111111", // System program
    //   bpfLoaderUpgradeableProgramAccount:
    //     "BPFLoaderUpgradeab1e11111111111111111111111", // BPF Loader Upgradeable
    //   maxDataLength: BigInt(programBuffer.length),
    // });

    // const transaction = createTransaction({
    //   version: "legacy",
    //   feePayer: deployerSigner.address,
    //   blockhash: latestBlockhash.blockhash,
    //   lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    //   instructions: [deployInstruction],
    // });

    // // Sign the transaction
    // console.log("✍️  Signing transaction...");
    // const signedTransaction =
    //   await signTransactionMessageWithSigners(transaction);

    // // Get transaction signature before sending
    // const signature = getSignatureFromTransaction(signedTransaction);
    // console.log(`Transaction signature: ${signature}`);

    // // Send and confirm transaction
    // console.log("📡 Sending transaction to network...");
    // const result = await sendAndConfirmTransaction(client, signedTransaction);

    // console.log("✅ Program deployed successfully!");
    // console.log(`Transaction signature: ${result.signature}`);
    // console.log(
    //   `Explorer: https://explorer.solana.com/tx/${result.signature}?cluster=devnet`,
    // );

    // return result.signature;
  } catch (error) {
    console.error("❌ Deployment failed:", error);
    throw error;
  }
}

// // Alternative method for upgrading an existing program
// async function upgradeProgram(programId: string) {
//   try {
//     console.log("🔄 Starting program upgrade...");

//     const rpc = createSolanaRpc(RPC_URL);
//     const client = createSolanaClient({ rpc });
//     const deployerSigner = await loadKeypairSignerFromFile(KEYPAIR_PATH);

//     // Read the new program binary
//     const programBuffer = readFileSync(path.resolve(PROGRAM_SO_PATH));

//     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

//     // Create upgrade instruction
//     const upgradeInstruction = createBpfLoaderUpgradeableUpgradeInstruction({
//       programDataAccount: programId, // The program data account
//       programAccount: programId,
//       bufferAccount: deployerSigner.address, // Buffer account with new program data
//       spillAccount: deployerSigner.address, // Account to receive refunded lamports
//       rentAccount: "11111111111111111111111111111111",
//       clockAccount: "SysvarC1ock11111111111111111111111111111111",
//       authorityAccount: deployerSigner.address, // Upgrade authority
//       bpfLoaderUpgradeableProgramAccount:
//         "BPFLoaderUpgradeab1e11111111111111111111111",
//     });

//     const transaction = createTransaction({
//       version: "legacy",
//       feePayer: deployerSigner.address,
//       blockhash: latestBlockhash.blockhash,
//       lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
//       instructions: [upgradeInstruction],
//     });

//     const signedTransaction =
//       await signTransactionMessageWithSigners(transaction);
//     const result = await sendAndConfirmTransaction(client, signedTransaction);

//     console.log("✅ Program upgraded successfully!");
//     console.log(`Transaction signature: ${result.signature}`);

//     return result.signature;
//   } catch (error) {
//     console.error("❌ Upgrade failed:", error);
//     throw error;
//   }
// }

// Main execution
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (command === "deploy") {
    await deployProgram();
  }
  // else if (command === "upgrade" && args[1]) {
  //   await upgradeProgram(args[1]);
  // }
  else {
    console.log("Usage:");
    console.log("  Deploy: npm run deploy");
    console.log("  Upgrade: npm run upgrade <program-id>");
  }
}

main();

// // Run the script
// if (require.main === module) {
//   main().catch(console.error);
// }

// export { deployProgram, upgradeProgram };
