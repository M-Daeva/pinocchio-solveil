import { PublicKey } from "@solana/web3.js";
import { N } from ".";

export interface InitArgs {
  dex: PublicKey;
  registry?: PublicKey;
  rotationTimeout?: N<32>;
}

export interface UpdateConfigArgs {
  admin?: PublicKey;
  dex?: PublicKey;
  registry?: PublicKey;
  isPaused?: boolean;
  rotationTimeout?: N<32>;
}

export interface SaveRouteArgs {
  mintFirst: PublicKey;
  mintLast: PublicKey;
  route: RouteItem[];
}

export interface RouteItem {
  ammIndex: N<16>;
  tokenOut: PublicKey;
}

export interface SwapArgs {
  tokenIn: PublicKey;
  tokenOut: PublicKey;
  amountIn: N<64>;
  amountOutMinimum: N<64>;
}
