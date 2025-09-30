import { IAssetItem, IRange } from "./types";
import { IConfig, IUserId } from "./accounts";
import { IUpdateConfigInstructionDataArgs } from "./instructions";
import {
  AssetItem,
  Config,
  Range,
  UpdateConfigInstructionDataArgs,
  UserId,
} from "../../codama";
import {
  DEFAULT_ADDRESS,
  TBitField,
  TBitFieldBuilder,
  TUint32,
  TUint64,
} from "../../../interfaces/primitives";

// TODO
// x?.admin || DEFAULT_ADDRESS
// x?.accountBump || 0

// type codec args should be optional
export function encAssetItem(x?: IAssetItem): AssetItem {
  return {
    amount: new TUint64(x?.amount).getRaw(),
    asset: x?.asset || DEFAULT_ADDRESS,
  };
}

export function decAssetItem(x?: AssetItem): IAssetItem {
  return {
    amount: new TUint64(x?.amount).get(),
    asset: x?.asset || DEFAULT_ADDRESS,
  };
}

export function encRange(x?: IRange): Range {
  return {
    min: new TUint32(x?.min).getRaw(),
    max: new TUint32(x?.max).getRaw(),
  };
}

export function decRange(x?: Range): IRange {
  return {
    min: new TUint32(x?.min).get(),
    max: new TUint32(x?.max).get(),
  };
}

// state codec args can't be optional
export function encConfig(x: IConfig): Config {
  return {
    flags: new TBitFieldBuilder().withBool(x.isPaused).build().getRaw(),
    admin: x.admin || DEFAULT_ADDRESS,
    rotationTimeout: new TUint32(x.rotationTimeout).getRaw(),
    registrationFee: encAssetItem(x.registrationFee),
    dataSizeRange: encRange(x.dataSizeRange),
  };
}

export function decConfig(x: Config): IConfig {
  return {
    isPaused: new TBitField(x.flags).get(),
    admin: x.admin || DEFAULT_ADDRESS,
    rotationTimeout: new TUint32(x.rotationTimeout).get(),
    registrationFee: decAssetItem(x.registrationFee),
    dataSizeRange: decRange(x.dataSizeRange),
  };
}

export function encUserId(x: IUserId): UserId {
  return {
    flags: new TBitFieldBuilder()
      .withBool(x.isOpen)
      .withBool(x.isActivated)
      .build()
      .getRaw(),
    id: new TUint32(x.id).getRaw(),
    accountBump: x.accountBump,
    rotationStateBump: x.rotationStateBump,
  };
}

export function decUserId(x: UserId): IUserId {
  return {
    isOpen: new TBitField(x.flags).get(0),
    isActivated: new TBitField(x.flags).get(1),
    id: new TUint32(x.id).get(),
    accountBump: x.accountBump,
    rotationStateBump: x.rotationStateBump,
  };
}

// we need only encoders for ixs
// ix codec args can't be optional
export function encUpdateConfigInstructionDataArgs(
  x: IUpdateConfigInstructionDataArgs,
): UpdateConfigInstructionDataArgs {
  return {
    flags: new TBitFieldBuilder()
      .withOptBool(x.isPaused)
      .withOptNonBool(x.admin)
      .withOptNonBool(x.rotationTimeout)
      .withOptNonBool(x.registrationFeeAmount)
      .withOptNonBool(x.dataSizeRange)
      .build()
      .getRaw(),
    admin: x.admin || DEFAULT_ADDRESS,
    rotationTimeout: new TUint32(x.rotationTimeout).getRaw(),
    registrationFeeAmount: new TUint64(x.registrationFeeAmount).getRaw(),
    dataSizeRange: encRange(x.dataSizeRange),
  };
}
