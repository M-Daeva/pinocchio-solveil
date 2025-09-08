import * as anchor from "@coral-xyz/anchor";
import { NetworkConfig, ProgramAddress } from "../interfaces";
import { PublicKey } from "@solana/web3.js";

export type ProgramName = "REGISTRY" | "DEX_ADAPTER";
export const networks = ["LOCALNET", "DEVNET", "MAINNET"] as const;

export const NETWORK_CONFIG: NetworkConfig = {
  LOCALNET: "http://localhost:8899",
  DEVNET: "https://api.devnet.solana.com",
  MAINNET: "https://api.mainnet-beta.solana.com",
};

export const COMMITMENT: anchor.web3.Commitment = "confirmed";

export const PATH = {
  TO_CONFIG: "./scripts/common/config/index.ts",
  OWNER_KEYPAIR: "../../.test-wallets/solana/dev-keypair.json",
};

export const UTILS = {
  MS_PER_SECOND: 1_000,
  ENCODING: "utf8",
};

export const REVENUE_MINT = {
  DEVNET: new PublicKey("fPcP9vGoowPikgu7oTRCJKHUvSNn9N5WZhYshR4UXyo"),
};
