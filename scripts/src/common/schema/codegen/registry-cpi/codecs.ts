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
  TAddress,
  TBitField,
  TBitFieldBuilder,
  TUint32,
  TUint64,
  TUint8,
} from "../../../interfaces/primitives";

// type codec args should be optional
export function encAssetItem(x?: IAssetItem): AssetItem {
  return {
    amount: new TUint64(x?.amount).getRaw(),
    asset: new TAddress(x?.asset).getRaw(),
  };
}

export function decAssetItem(x?: AssetItem): IAssetItem {
  return {
    amount: new TUint64(x?.amount).get(),
    asset: new TAddress(x?.asset).get(),
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
    admin: new TAddress(x.admin).getRaw(),
    rotationTimeout: new TUint32(x.rotationTimeout).getRaw(),
    registrationFee: encAssetItem(x.registrationFee),
    dataSizeRange: encRange(x.dataSizeRange),
  };
}

export function decConfig(x: Config): IConfig {
  return {
    isPaused: new TBitField(x.flags).get(),
    admin: new TAddress(x.admin).get(),
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
    accountBump: new TUint8(x.accountBump).getRaw(),
    rotationStateBump: new TUint8(x.rotationStateBump).getRaw(),
  };
}

export function decUserId(x: UserId): IUserId {
  return {
    isOpen: new TBitField(x.flags).get(0),
    isActivated: new TBitField(x.flags).get(1),
    id: new TUint32(x.id).get(),
    accountBump: new TUint8(x.accountBump).get(),
    rotationStateBump: new TUint8(x.rotationStateBump).get(),
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
    admin: new TAddress(x.admin).get(),
    rotationTimeout: new TUint32(x.rotationTimeout).getRaw(),
    registrationFeeAmount: new TUint64(x.registrationFeeAmount).getRaw(),
    dataSizeRange: encRange(x.dataSizeRange),
  };
}
