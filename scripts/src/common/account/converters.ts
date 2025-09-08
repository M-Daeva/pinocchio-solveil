import { gcmsiv } from "@noble/ciphers/aes";
import * as base64js from "base64-js";

const ENC_KEY_LEN = 32;

export function toBase64(data: Uint8Array): string {
  return base64js.fromByteArray(data);
}

export function fromBase64(base64String: string): Uint8Array {
  if (!base64String.match(/^[a-zA-Z0-9+/]*={0,2}$/)) {
    throw new Error("Invalid base64 string format");
  }
  return base64js.toByteArray(base64String);
}

export function toHex(data: Uint8Array): string {
  let out = "";
  for (const byte of data) {
    out += ("0" + byte.toString(16)).slice(-2);
  }
  return out;
}

export function fromHex(hexstring: string): Uint8Array {
  if (hexstring.length % 2 !== 0) {
    throw new Error("hex string length must be a multiple of 2");
  }

  const out = new Uint8Array(hexstring.length / 2);
  for (let i = 0; i < out.length; i++) {
    const j = 2 * i;
    const hexByteAsString = hexstring.slice(j, j + 2);
    if (!hexByteAsString.match(/[0-9a-f]{2}/i)) {
      throw new Error("hex string contains invalid characters");
    }
    out[i] = parseInt(hexByteAsString, 16);
  }
  return out;
}

export interface EncryptedResponse {
  timestamp: string;
  value: string;
}

export function addressToSalt(address: string): string {
  return address.repeat(2);
}

export function generateBindingHashPassword(
  timestamp: string,
  account: string
): string {
  const randomDecimal = Math.random();
  return `${timestamp}${randomDecimal}${account}`;
}

function timestampToNonce(timestamp: string): string {
  return timestamp.slice(0, 12);
}

function getCipher(encKey: Uint8Array, nonce: Uint8Array) {
  if (encKey.length !== ENC_KEY_LEN) {
    throw new Error(`Encryption key must be ${ENC_KEY_LEN} bytes`);
  }
  return gcmsiv(encKey, nonce);
}

function encrypt(msg: string, encKey: Uint8Array, nonce: string): string {
  const nonceBytes = new TextEncoder().encode(nonce);
  const msgBytes = new TextEncoder().encode(msg);
  const cipher = getCipher(encKey, nonceBytes);

  const encrypted = cipher.encrypt(msgBytes);
  return toBase64(encrypted);
}

function decrypt(encMsg: string, encKey: Uint8Array, nonce: string): string {
  const nonceBytes = new TextEncoder().encode(nonce);
  const cipher = getCipher(encKey, nonceBytes);
  const msgBytes = fromBase64(encMsg);

  const decrypted = cipher.decrypt(msgBytes);
  return new TextDecoder().decode(decrypted);
}

function serialize<T>(data: T): string {
  return JSON.stringify(data);
}

function deserialize<T>(data: string): T {
  return JSON.parse(data) as T;
}

export function serializeEncrypt<T>(
  encKey: string, // hex string
  timestamp: string,
  value: T
): EncryptedResponse {
  const key = fromHex(encKey);

  // Ensure key length
  if (key.length !== ENC_KEY_LEN) {
    throw new Error(`Encryption key must be ${ENC_KEY_LEN} bytes`);
  }

  const nonce = timestampToNonce(timestamp);
  const serializedValue = serialize(value);
  const encryptedValue = encrypt(serializedValue, key, nonce);

  return {
    value: encryptedValue,
    timestamp: timestamp.toString(),
  };
}

export function decryptDeserialize<T>(
  encKey: string, // hex string
  timestamp: string,
  value: string
): T {
  const key = fromHex(encKey);

  // Ensure key length
  if (key.length !== ENC_KEY_LEN) {
    throw new Error(`Encryption key must be ${ENC_KEY_LEN} bytes`);
  }

  const nonce = timestampToNonce(timestamp);
  const decryptedData = decrypt(value, key, nonce);

  return deserialize<T>(decryptedData);
}
