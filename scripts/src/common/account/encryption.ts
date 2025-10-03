import { Address, createSignableMessage, KeyPairSigner } from "gill";
import { hkdf } from "@noble/hashes/hkdf";
import { sha256 } from "@noble/hashes/sha2";
import { toHex } from "./converters";

// Step 1: Create a deterministic message for signing
export function createDeterministicMessage(
  publicKey: Address,
  context: string = "password-manager",
): Uint8Array {
  const message = `${context}:${publicKey.toString()}`;
  return new TextEncoder().encode(message);
}

// Step 2: Request signature from wallet (using KeyPairSigner)
export async function requestSignature(
  signer: KeyPairSigner,
  message: Uint8Array,
): Promise<Uint8Array> {
  // KeyPairSigner has built-in message signing
  const [signatureDictionary] = await signer.signMessages([
    createSignableMessage(message),
  ]);

  // Extract the signature for this signer's address from the dictionary
  const signature = signatureDictionary?.[signer.address];

  if (!signature) {
    throw new Error("Failed to get signature from signer");
  }

  return signature;
}

// Step 3: Derive encryption key from signature
export function deriveEncryptionKey(
  signature: Uint8Array,
  publicKey: Address,
): Uint8Array {
  const CONTEXT = "solana-password-manager";

  // Create deterministic but unique salt per user
  const saltInput = `${CONTEXT}:salt:${publicKey.toString()}`;
  const salt = sha256(new TextEncoder().encode(saltInput));

  // Use HKDF for proper key derivation
  const encryptionKey = hkdf(
    sha256,
    signature, // High-entropy input from wallet signature
    salt, // Unique per user
    CONTEXT, // Application context
    32, // AES-256 key length
  );

  return encryptionKey;
}

// Complete flow
export async function generateEncryptionKey(
  signer: KeyPairSigner,
): Promise<string> {
  const message = createDeterministicMessage(signer.address);
  const signature = await requestSignature(signer, message);
  const encryptionKey = deriveEncryptionKey(signature, signer.address);

  return toHex(encryptionKey);
}

// async function main() {
//   const sender = await readKeypairSigner(PATH.OWNER_KEYPAIR);
//   const encryptionKey = await generateEncryptionKey(sender);
// }

// main();
