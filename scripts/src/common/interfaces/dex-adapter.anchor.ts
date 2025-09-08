// Auto-generated Anchor types and converters
import * as anchor from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import { InitArgs, UpdateConfigArgs, SaveRouteArgs, RouteItem, SwapArgs } from './dex-adapter';

// Anchor-generated types
export type AnchorInitArgs = [
  PublicKey,
  PublicKey | null,
  number | null
];

export type AnchorUpdateConfigArgs = [
  PublicKey | null,
  PublicKey | null,
  PublicKey | null,
  boolean | null,
  number | null
];

export type AnchorSaveRouteArgs = [
  PublicKey,
  PublicKey,
  AnchorRouteItem[]
];

export interface AnchorRouteItem {
  ammIndex: number;
  tokenOut: PublicKey;
}

export type AnchorSwapArgs = [
  PublicKey,
  PublicKey,
  anchor.BN,
  anchor.BN
];


// Type converters
export function convertInitArgs(
  args: InitArgs
): AnchorInitArgs {
  return [
    args.dex,
    args.registry !== undefined ? args.registry : null,
    args.rotationTimeout !== undefined ? args.rotationTimeout : null
  ];
}

export function convertUpdateConfigArgs(
  args: UpdateConfigArgs
): AnchorUpdateConfigArgs {
  return [
    args.admin !== undefined ? args.admin : null,
    args.dex !== undefined ? args.dex : null,
    args.registry !== undefined ? args.registry : null,
    args.isPaused !== undefined ? args.isPaused : null,
    args.rotationTimeout !== undefined ? args.rotationTimeout : null
  ];
}

export function convertSaveRouteArgs(
  args: SaveRouteArgs
): AnchorSaveRouteArgs {
  return [
    args.mintFirst,
    args.mintLast,
    args.route.map(convertRouteItem)
  ];
}

export function convertRouteItem(
  obj: RouteItem
): AnchorRouteItem {
  return {
    ammIndex: obj.ammIndex,
    tokenOut: obj.tokenOut,
  };
}

export function convertSwapArgs(
  args: SwapArgs
): AnchorSwapArgs {
  return [
    args.tokenIn,
    args.tokenOut,
    new anchor.BN(args.amountIn),
    new anchor.BN(args.amountOutMinimum)
  ];
}

