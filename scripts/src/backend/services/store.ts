import { loadKeypairSignerFromFile } from "gill/node";
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
  Instruction,
  InstructionWithAccounts,
  InstructionWithData,
  InstructionWithSigners,
  Address,
  AccountRole,
  generateKeyPairSigner,
} from "gill";
import { rootPath } from "../utils";
import { PATH } from "../../common/config";
import { l, li, numberFrom } from "../../common/utils";
import { readFile } from "fs/promises";

// Configuration
const RPC_URL = "https://api.devnet.solana.com"; // Change to mainnet for production
const PROGRAM_SO_PATH = "./target/deploy/your_program.so"; // Path to your compiled program
const KEYPAIR_PATH = "~/.config/solana/id.json"; // Path to your wallet keypair

// BPF Loader Upgradeable program ID
const BPF_LOADER_UPGRADEABLE_PROGRAM_ID =
  "BPFLoaderUpgradeab1e11111111111111111111111" as Address;

// System program and sysvar addresses
const SYSTEM_PROGRAM_ID = "11111111111111111111111111111111" as Address;
const RENT_SYSVAR_ID = "SysvarRent111111111111111111111111111111111" as Address;
const CLOCK_SYSVAR_ID =
  "SysvarC1ock11111111111111111111111111111111" as Address;

// BPF Loader Upgradeable instruction discriminators
const BPF_LOADER_UPGRADEABLE_INSTRUCTIONS = {
  INITIALIZE_BUFFER: 0,
  WRITE: 1,
  DEPLOY_WITH_MAX_DATA_LEN: 2,
  UPGRADE: 3,
  SET_AUTHORITY: 4,
  CLOSE: 5,
} as const;

// Helper function to create BPF Loader Upgradeable deploy instruction
function createBpfLoaderUpgradeableDeployInstruction(params: {
  payerAccount: Address;
  programDataAccount: Address;
  programAccount: Address;
  bufferAccount: Address;
  rentSysvarAccount: Address;
  clockSysvarAccount: Address;
  systemProgramAccount: Address;
  authorityAccount?: Address;
  maxDataLength: bigint;
}): Instruction<typeof BPF_LOADER_UPGRADEABLE_PROGRAM_ID> {
  // Create instruction data
  const instructionData = new Uint8Array(12); // 4 bytes discriminator + 8 bytes max_data_len
  const view = new DataView(instructionData.buffer);

  // Set instruction discriminator (DEPLOY_WITH_MAX_DATA_LEN = 2)
  view.setUint32(
    0,
    BPF_LOADER_UPGRADEABLE_INSTRUCTIONS.DEPLOY_WITH_MAX_DATA_LEN,
    true,
  );

  // Set max data length (8 bytes, little-endian)
  view.setBigUint64(4, params.maxDataLength, true);

  return {
    programAddress: BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
    data: instructionData,
    accounts: [
      { address: params.payerAccount, role: AccountRole.WRITABLE_SIGNER },
      { address: params.programDataAccount, role: AccountRole.WRITABLE },
      { address: params.programAccount, role: AccountRole.WRITABLE },
      { address: params.bufferAccount, role: AccountRole.WRITABLE },
      { address: params.rentSysvarAccount, role: AccountRole.READONLY },
      { address: params.clockSysvarAccount, role: AccountRole.READONLY },
      { address: params.systemProgramAccount, role: AccountRole.READONLY },
      ...(params.authorityAccount
        ? [
            {
              address: params.authorityAccount,
              role: AccountRole.READONLY_SIGNER,
            },
          ]
        : []),
    ],
  };
}

// Helper function to create write buffer instruction
function createBpfLoaderUpgradeableWriteInstruction(params: {
  bufferAccount: Address;
  authorityAccount: Address;
  offset: number;
  data: Uint8Array;
}): Instruction<typeof BPF_LOADER_UPGRADEABLE_PROGRAM_ID> {
  // Create instruction data: discriminator (4 bytes) + offset (4 bytes) + data
  const instructionData = new Uint8Array(8 + params.data.length);
  const view = new DataView(instructionData.buffer);

  // Set instruction discriminator (WRITE = 1)
  view.setUint32(0, BPF_LOADER_UPGRADEABLE_INSTRUCTIONS.WRITE, true);

  // Set offset
  view.setUint32(4, params.offset, true);

  // Copy data
  instructionData.set(params.data, 8);

  return {
    programAddress: BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
    data: instructionData,
    accounts: [
      { address: params.bufferAccount, role: AccountRole.WRITABLE },
      { address: params.authorityAccount, role: AccountRole.READONLY_SIGNER },
    ],
  };
}

// Helper function to create initialize buffer instruction
function createBpfLoaderUpgradeableInitializeBufferInstruction(params: {
  bufferAccount: Address;
  authorityAccount: Address;
}): Instruction<typeof BPF_LOADER_UPGRADEABLE_PROGRAM_ID> {
  // Create instruction data with discriminator and authority pubkey
  const instructionData = new Uint8Array(36); // 4 bytes discriminator + 32 bytes pubkey
  const view = new DataView(instructionData.buffer);

  // Set instruction discriminator (INITIALIZE_BUFFER = 0)
  view.setUint32(
    0,
    BPF_LOADER_UPGRADEABLE_INSTRUCTIONS.INITIALIZE_BUFFER,
    true,
  );

  // TODO: Add authority pubkey bytes starting at offset 4
  // For now, we'll leave it as zeros which means no authority

  return {
    programAddress: BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
    data: instructionData,
    accounts: [
      { address: params.bufferAccount, role: AccountRole.WRITABLE },
      { address: RENT_SYSVAR_ID, role: AccountRole.READONLY },
    ],
  };
}

async function deployProgram() {
  // Configuration
  const RPC_URL = "https://api.devnet.solana.com"; // Change to mainnet for production
  const PROGRAM_SO_PATH = "../target/deploy/registry.so"; // Path to your compiled program

  try {
    l("🚀 Starting Solana program deployment with Gill...");

    // Create RPC connection
    const rpc = createSolanaRpc(RPC_URL);
    // const client = createSolanaClient({ rpc });
    const client = createSolanaClient({ urlOrMoniker: "devnet" });

    // TODO: client.simulateTransaction instead of getSimulationComputeUnits
    // import {ComputeBudgetInstruction} from "gill/programs"

    // Load the deployer keypair
    l("📝 Loading keypair...");
    const deployerSigner = await loadKeypairSignerFromFile(
      rootPath(PATH.OWNER_KEYPAIR),
    );
    l(`Deployer address: ${deployerSigner.address}`);

    // Check balance
    const balance = numberFrom(
      (await rpc.getBalance(deployerSigner.address).send()).value,
    )
      .div(LAMPORTS_PER_SOL)
      .toDecimalPlaces(3)
      .toNumber();
    l(`Deployer balance: ${balance} SOL`);

    if (balance < 0.001) {
      throw new Error(
        "Insufficient balance. Need at least 0.001 SOL for deployment.",
      );
    }

    // Read the program binary
    l("📖 Reading program binary...");
    const programBuffer = await readFile(path.resolve(PROGRAM_SO_PATH));
    l(`Program size: ${programBuffer.length} bytes`);

    // Calculate the minimum balance needed for the program account
    // const programAccountSpace = BigInt(programBuffer.length);
    // const minimumBalanceForRentExemption = await rpc
    //   .getMinimumBalanceForRentExemption(programAccountSpace)
    //   .send();

    // l(
    //   `Min Rent Exemption: ${numberFrom(minimumBalanceForRentExemption).div(LAMPORTS_PER_SOL).toFixed(3)} lamports`,
    // );

    // Create program deployment transaction
    l("🔨 Creating deployment transaction...");

    // Generate keypairs for program accounts
    l("🔑 Generating program keypairs...");
    const programSigner = await generateKeyPairSigner();
    const programDataSigner = await generateKeyPairSigner();
    const bufferSigner = await generateKeyPairSigner();

    l(`Program ID: ${programSigner.address}`);
    l(`Program Data Account: ${programDataSigner.address}`);
    l(`Buffer Account: ${bufferSigner.address}`);

    // Get recent blockhash
    l("🔗 Getting recent blockhash...");
    const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    // Calculate required account sizes and rent
    const bufferAccountSize = programBuffer.length + 37; // Buffer header is 37 bytes
    const programDataAccountSize = programBuffer.length + 45; // ProgramData header is 45 bytes
    const programAccountSize = 36; // Program account is 36 bytes

    const bufferRent = await rpc
      .getMinimumBalanceForRentExemption(BigInt(bufferAccountSize))
      .send();
    const programDataRent = numberFrom(
      await rpc
        .getMinimumBalanceForRentExemption(BigInt(programDataAccountSize))
        .send(),
    );
    const programRent = numberFrom(
      await rpc
        .getMinimumBalanceForRentExemption(BigInt(programAccountSize))
        .send(),
    );

    l(
      `Buffer rent: ${numberFrom(bufferRent).div(LAMPORTS_PER_SOL).toFixed(3)} SOL`,
    );
    l(
      `Program data rent: ${programDataRent.div(LAMPORTS_PER_SOL).toFixed(3)} SOL`,
    );
    l(`Program rent: ${programRent.div(LAMPORTS_PER_SOL).toFixed(3)} SOL`);

    const instructions: Instruction[] = [];

    // Step 1: Create buffer account
    instructions.push({
      programAddress: SYSTEM_PROGRAM_ID,
      data: new Uint8Array([
        0,
        0,
        0,
        0, // CreateAccount discriminator
        ...new Uint8Array(new BigUint64Array([bufferRent]).buffer),
        ...new Uint8Array(
          new BigUint64Array([BigInt(bufferAccountSize)]).buffer,
        ),
        ...Array.from(
          { length: 32 },
          (_, i) => BPF_LOADER_UPGRADEABLE_PROGRAM_ID.charCodeAt(i) || 0,
        ),
      ]),
      accounts: [
        { address: deployerSigner.address, role: AccountRole.WRITABLE_SIGNER },
        { address: bufferSigner.address, role: AccountRole.WRITABLE_SIGNER },
      ],
    });

    // Step 2: Initialize buffer
    instructions.push(
      createBpfLoaderUpgradeableInitializeBufferInstruction({
        bufferAccount: bufferSigner.address,
        authorityAccount: deployerSigner.address,
      }),
    );

    // Step 3: Write program data to buffer (in chunks if necessary)
    const maxChunkSize = 800; // Keep chunks small to avoid transaction size limits
    for (
      let offset = 0;
      offset < programBuffer.length;
      offset += maxChunkSize
    ) {
      const chunk = programBuffer.slice(offset, offset + maxChunkSize);
      instructions.push(
        createBpfLoaderUpgradeableWriteInstruction({
          bufferAccount: bufferSigner.address,
          authorityAccount: deployerSigner.address,
          offset,
          data: chunk,
        }),
      );
    }

    // Step 4: Deploy the program
    instructions.push(
      createBpfLoaderUpgradeableDeployInstruction({
        payerAccount: deployerSigner.address,
        programDataAccount: programDataSigner.address,
        programAccount: programSigner.address,
        bufferAccount: bufferSigner.address,
        rentSysvarAccount: RENT_SYSVAR_ID,
        clockSysvarAccount: CLOCK_SYSVAR_ID,
        systemProgramAccount: SYSTEM_PROGRAM_ID,
        authorityAccount: deployerSigner.address,
        maxDataLength: BigInt(programBuffer.length * 2), // Allow for future upgrades
      }),
    );

    // Create transaction
    l("🔨 Creating deployment transaction...");

    // TODO: set CU
    const transaction = createTransaction({
      version: "legacy",
      feePayer: deployerSigner.address,
      latestBlockhash,
      instructions: instructions.slice(0, 3), // Start with first few instructions
    });

    // Sign the transaction with all required signers
    l("✍️  Signing transaction...");
    const signedTransaction =
      await signTransactionMessageWithSigners(transaction);

    // signers: [
    //   deployerSigner,
    //   bufferSigner,
    //   programSigner,
    //   programDataSigner,
    // ],

    // Get transaction signature before sending
    const signature = getSignatureFromTransaction(signedTransaction);
    l(`Transaction signature: ${signature}`);

    // // Send and confirm transaction
    // l("📡 Sending transaction to network...");
    // const result = await sendAndConfirmTransaction(client, signedTransaction);

    // l("✅ Program deployed successfully!");
    // l(`Program ID: ${programSigner.address}`);
    // l(`Transaction signature: ${result.signature}`);
    // l(
    //   `Explorer: https://explorer.solana.com/tx/${result.signature}?cluster=devnet`,
    // );

    // return {
    //   programId: programSigner.address,
    //   signature: result.signature,
    // };
  } catch (error) {
    console.error("❌ Deployment failed:", error);
    throw error;
  }
}

// // Alternative method for upgrading an existing program
// async function upgradeProgram(programId: string) {
//   try {
//     l("🔄 Starting program upgrade...");

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

//     l("✅ Program upgraded successfully!");
//     l(`Transaction signature: ${result.signature}`);

//     return result.signature;
//   } catch (error) {
//     console.error("❌ Upgrade failed:", error);
//     throw error;
//   }
// }

// Main execution
async function main() {
  await deployProgram();
}

main();

// // Run the script
// if (require.main === module) {
//   main().catch(console.error);
// }

// export { deployProgram, upgradeProgram };
