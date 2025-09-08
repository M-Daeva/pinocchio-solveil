import { PublicKey, Keypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import nacl from "tweetnacl";
import { hkdf } from "@noble/hashes/hkdf";
import { sha256 } from "@noble/hashes/sha2";
import { toHex } from "./converters";

// anchor.Wallet can't sign messages, that's why we need extended class for Nodejs (FE wallets should be fine)
export class MessageSigningWallet extends anchor.Wallet {
  constructor(readonly payer: Keypair) {
    super(payer);
  }

  async signMessage(message: Uint8Array): Promise<Uint8Array> {
    const signature = nacl.sign.detached(message, this.payer.secretKey);
    return new Uint8Array(signature);
  }

  get publicKey(): PublicKey {
    return this.payer.publicKey;
  }
}

export function getSigningWallet(
  ownerKeypair: anchor.web3.Keypair
): MessageSigningWallet {
  return new MessageSigningWallet(ownerKeypair);
}

// Step 1: Create a deterministic message for signing
export function createDeterministicMessage(
  publicKey: PublicKey,
  context: string = "password-manager"
): Uint8Array {
  const message = `${context}:${publicKey.toString()}`;
  return new TextEncoder().encode(message);
}

// Step 2: Request signature from wallet (using wallet adapter)
export async function requestSignature(
  wallet: MessageSigningWallet,
  message: Uint8Array
): Promise<Uint8Array> {
  // This will prompt the user to sign with their wallet
  const signature = await wallet.signMessage(message);
  return signature;
}

// Step 3: Derive encryption key from signature
export function deriveEncryptionKey(
  signature: Uint8Array,
  publicKey: PublicKey
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
    32 // AES-256 key length
  );

  return encryptionKey;
}

// Complete flow
export async function generateEncryptionKey(
  wallet: MessageSigningWallet
): Promise<string> {
  const message = createDeterministicMessage(wallet.publicKey);
  const signature = await requestSignature(wallet, message);
  const encryptionKey = deriveEncryptionKey(signature, wallet.publicKey);

  return toHex(encryptionKey);
}

// async function main() {
//   const ownerKeypair = await readKeypair(rootPath(PATH.OWNER_KEYPAIR));
//   const wallet = getSigningWallet(ownerKeypair);

//   const encryptionKey = await generateEncryptionKey(wallet);
// }

// main();
