import path from "path";
import { getAddressEncoder } from "@solana/addresses";
import { rootPath } from "../utils";
import { PATH } from "../../common/config";
import { l, li, numberFrom } from "../../common/utils";
import { readFile } from "fs/promises";
import bs58 from "bs58";
import { loadKeypairSignerFromFile } from "gill/node";
import {
  createSolanaRpc,
  createSolanaClient,
  createTransaction,
  LAMPORTS_PER_SOL,
  Instruction,
  Address,
  AccountRole,
  generateKeyPairSigner,
  compileTransaction,
  signTransaction,
  SimulateTransactionFunction,
  CompilableTransactionMessage,
  SignaturesMap,
  TransactionMessageBytes,
} from "gill";

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

async function simulateTx(
  tx:
    | Readonly<{
        messageBytes: TransactionMessageBytes;
        signatures: SignaturesMap;
      }>
    | CompilableTransactionMessage,
  simulateTransaction: SimulateTransactionFunction,
) {
  const simulationResult = await simulateTransaction(tx, {
    commitment: "confirmed",
  });

  l("Simulation result:", simulationResult);

  if (simulationResult.value.err) {
    l("Simulation failed:", simulationResult.value.err);
    l("Logs:", simulationResult.value.logs);
  }
}

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
      // Authority should be a signer for the deploy instruction
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
  // The InitializeBuffer instruction format:
  // 4 bytes: instruction discriminator
  // 1 byte: option (0 = None, 1 = Some)
  // 32 bytes: authority pubkey (if option = 1)
  const instructionData = new Uint8Array(37);
  const view = new DataView(instructionData.buffer);

  // Set instruction discriminator (INITIALIZE_BUFFER = 0)
  view.setUint32(
    0,
    BPF_LOADER_UPGRADEABLE_INSTRUCTIONS.INITIALIZE_BUFFER,
    true,
  );

  // Set option byte to 1 (Some) for the authority
  instructionData[4] = 1;

  // Convert address string to raw bytes
  // Solana addresses are base58 encoded, we need to decode them
  const authorityBytes = bs58.decode(params.authorityAccount);

  // Copy the 32 bytes of the authority pubkey
  instructionData.set(authorityBytes, 5);

  return {
    programAddress: BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
    data: instructionData,
    accounts: [
      { address: params.bufferAccount, role: AccountRole.WRITABLE },
      { address: RENT_SYSVAR_ID, role: AccountRole.READONLY },
    ],
  };
}

// Helper function to create system program create account instruction
function createSystemCreateAccountInstruction(params: {
  fromAccount: Address;
  toAccount: Address;
  lamports: bigint;
  space: bigint;
  owner: Address;
}): Instruction<typeof SYSTEM_PROGRAM_ID> {
  // Create instruction data for CreateAccount
  const instructionData = new Uint8Array(52); // 4 bytes discriminator + 8 bytes lamports + 8 bytes space + 32 bytes owner
  const view = new DataView(instructionData.buffer);

  // Set instruction discriminator (CreateAccount = 0)
  view.setUint32(0, 0, true);

  // Set lamports
  view.setBigUint64(4, params.lamports, true);

  // Set space
  view.setBigUint64(12, params.space, true);

  // Set owner (32 bytes)
  const addressEncoder = getAddressEncoder();
  const ownerBytes = addressEncoder.encode(params.owner);
  instructionData.set(ownerBytes, 20);

  return {
    programAddress: SYSTEM_PROGRAM_ID,
    data: instructionData,
    accounts: [
      { address: params.fromAccount, role: AccountRole.WRITABLE_SIGNER },
      { address: params.toAccount, role: AccountRole.WRITABLE_SIGNER },
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
    const { sendAndConfirmTransaction, simulateTransaction } =
      createSolanaClient({
        urlOrMoniker: "devnet",
      });

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

    if (balance < 1) {
      throw new Error(
        "Insufficient balance. Need at least 1 SOL for deployment.",
      );
    }

    // Read the program binary
    l("📖 Reading program binary...");
    const programBuffer = await readFile(path.resolve(PROGRAM_SO_PATH));
    l(`Program size: ${programBuffer.length} bytes`);

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
    const programDataRent = await rpc
      .getMinimumBalanceForRentExemption(BigInt(programDataAccountSize))
      .send();
    const programRent = await rpc
      .getMinimumBalanceForRentExemption(BigInt(programAccountSize))
      .send();

    l(
      `Buffer rent: ${numberFrom(bufferRent).div(LAMPORTS_PER_SOL).toFixed(3)} SOL`,
    );
    l(
      `Program data rent: ${numberFrom(programDataRent).div(LAMPORTS_PER_SOL).toFixed(3)} SOL`,
    );
    l(
      `Program rent: ${numberFrom(programRent).div(LAMPORTS_PER_SOL).toFixed(3)} SOL`,
    );

    // Step 1: Create and initialize buffer account
    l("🔨 Creating buffer creation transaction...");

    const bufferCreationInstructions = [
      createSystemCreateAccountInstruction({
        fromAccount: deployerSigner.address,
        toAccount: bufferSigner.address,
        lamports: bufferRent,
        space: BigInt(bufferAccountSize),
        owner: BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
      }),
      createBpfLoaderUpgradeableInitializeBufferInstruction({
        bufferAccount: bufferSigner.address,
        authorityAccount: deployerSigner.address,
      }),
    ];

    const bufferCreationTransaction = createTransaction({
      version: "legacy",
      feePayer: deployerSigner.address,
      latestBlockhash,
      instructions: bufferCreationInstructions,
    });

    // Compile the transaction to get messageBytes and signatures
    l("📦 Compiling buffer creation transaction...");
    const compiledBufferTransaction = compileTransaction(
      bufferCreationTransaction,
    );

    // Manually sign with required signers
    l("✍️  Signing buffer creation transaction...");
    const signedTransaction = await signTransaction(
      [bufferSigner.keyPair, deployerSigner.keyPair],
      compiledBufferTransaction,
    );

    await simulateTx(signedTransaction, simulateTransaction);
    const bufferResult = await sendAndConfirmTransaction(signedTransaction);
    l(`Buffer creation transaction: ${bufferResult}`);

    // Step 2: Write program data to buffer in smaller chunks
    l("📝 Writing program data to buffer...");

    // Optimize chunk size - make it larger but still safe
    const maxChunkSize = 800; // Increased from 400 but still conservative
    const maxInstructionsPerTx = 1; // Keep at 1 instruction per transaction for safety

    for (
      let offset = 0;
      offset < programBuffer.length;
      offset += maxChunkSize
    ) {
      const chunk = programBuffer.slice(offset, offset + maxChunkSize);

      // Create a single write instruction
      const writeInstruction = createBpfLoaderUpgradeableWriteInstruction({
        bufferAccount: bufferSigner.address,
        authorityAccount: deployerSigner.address,
        offset,
        data: chunk,
      });

      // Get fresh blockhash for each transaction
      const { value: freshBlockhash } = await rpc.getLatestBlockhash().send();

      const writeTransaction = createTransaction({
        version: "legacy",
        feePayer: deployerSigner.address,
        latestBlockhash: freshBlockhash,
        instructions: [writeInstruction], // Only one instruction per transaction
      });

      const signedWrite = await signTransaction(
        [deployerSigner.keyPair],
        compileTransaction(writeTransaction),
      );

      await simulateTx(signedWrite, simulateTransaction);
      const writeResult = await sendAndConfirmTransaction(signedWrite);

      const chunkNumber = Math.floor(offset / maxChunkSize) + 1;
      const totalChunks = Math.ceil(programBuffer.length / maxChunkSize);
      l(
        `Write chunk ${chunkNumber}/${totalChunks} (offset: ${offset}, size: ${chunk.length}) transaction: ${writeResult}`,
      );

      // Add a small delay to avoid overwhelming the RPC
      await new Promise((resolve) => setTimeout(resolve, 500));
    }

    // Step 3: Create program and program data accounts
    l("🏗️  Creating program accounts...");
    const { value: deployBlockhash } = await rpc.getLatestBlockhash().send();

    const accountCreationInstructions = [
      createSystemCreateAccountInstruction({
        fromAccount: deployerSigner.address,
        toAccount: programSigner.address,
        lamports: programRent,
        space: BigInt(programAccountSize),
        owner: BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
      }),
      createSystemCreateAccountInstruction({
        fromAccount: deployerSigner.address,
        toAccount: programDataSigner.address,
        lamports: programDataRent,
        space: BigInt(programDataAccountSize),
        owner: BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
      }),
    ];

    const accountCreationTransaction = createTransaction({
      version: "legacy",
      feePayer: deployerSigner.address,
      latestBlockhash: deployBlockhash,
      instructions: accountCreationInstructions,
    });

    // Sign with all required signers
    const fullySignedAccounts = await signTransaction(
      [
        programSigner.keyPair,
        programDataSigner.keyPair,
        deployerSigner.keyPair,
      ],
      compileTransaction(accountCreationTransaction),
    );

    await simulateTx(fullySignedAccounts, simulateTransaction);
    const accountCreationResult =
      await sendAndConfirmTransaction(fullySignedAccounts);
    l(`Account creation transaction: ${accountCreationResult}`);

    // Step 4: Deploy the program
    l("🚀 Deploying program...");
    const { value: finalBlockhash } = await rpc.getLatestBlockhash().send();

    const deployInstruction = createBpfLoaderUpgradeableDeployInstruction({
      payerAccount: deployerSigner.address,
      programDataAccount: programDataSigner.address,
      programAccount: programSigner.address,
      bufferAccount: bufferSigner.address,
      rentSysvarAccount: RENT_SYSVAR_ID,
      clockSysvarAccount: CLOCK_SYSVAR_ID,
      systemProgramAccount: SYSTEM_PROGRAM_ID,
      authorityAccount: deployerSigner.address,
      maxDataLength: BigInt(programBuffer.length * 2), // Allow for future upgrades
    });

    // TODO: set CU
    const deployTransaction = createTransaction({
      version: "legacy",
      feePayer: deployerSigner.address,
      latestBlockhash: finalBlockhash,
      instructions: [deployInstruction],
    });

    const signedDeploy = await signTransaction(
      [deployerSigner.keyPair],
      compileTransaction(deployTransaction),
    );

    await simulateTx(signedDeploy, simulateTransaction);
    const deployResult = await sendAndConfirmTransaction(signedDeploy);

    l("✅ Program deployed successfully!");
    l(`Program ID: ${programSigner.address}`);
    l(`Final transaction signature: ${deployResult}`);
    l(
      `Explorer: https://explorer.solana.com/tx/${deployResult}?cluster=devnet`,
    );

    return {
      programId: programSigner.address,
      signature: deployResult,
    };
  } catch (error) {
    console.error("❌ Deployment failed:", error);
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

// export { deployProgram, upgradeProgram };
