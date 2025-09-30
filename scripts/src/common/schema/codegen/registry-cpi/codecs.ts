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
  FlagsHandler,
  TBitField,
  TUint32,
  TUint64,
} from "../../../interfaces/primitives";

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

// type/state codecs args should be optional
export function encConfig(x?: IConfig): Config {
  return {
    flags: new TBitField({ flag: x?.isPaused || false }).getRaw(),
    admin: x?.admin || DEFAULT_ADDRESS,
    rotationTimeout: new TUint32(x?.rotationTimeout).getRaw(),
    registrationFee: encAssetItem(x?.registrationFee),
    dataSizeRange: encRange(x?.dataSizeRange),
  };
}

export function decConfig(x?: Config): IConfig {
  return {
    isPaused: new TBitField({ value: x?.flags || 0 }).get(),
    admin: x?.admin || DEFAULT_ADDRESS,
    rotationTimeout: new TUint32(x?.rotationTimeout).get(),
    registrationFee: decAssetItem(x?.registrationFee),
    dataSizeRange: decRange(x?.dataSizeRange),
  };
}

// TODO
// export function encUserId(x?: IUserId): UserId {
//   return {
//     flags,
//     id,
//     accountBump,
//     rotationStateBump,
//   };
// }

// we need only encoders for ixs
// ix codecs args can't be optional
export function encUpdateConfigInstructionDataArgs(
  x: IUpdateConfigInstructionDataArgs,
): UpdateConfigInstructionDataArgs {
  // TODO: add optional boolean feature
  const fh = new FlagsHandler();

  let data: UpdateConfigInstructionDataArgs = {
    flags: 0,
    admin: fh.add(x.admin, DEFAULT_ADDRESS),
    // isPaused: new TBitField({ flag: fh.add(x.isPaused, false) }).getRaw(),
    rotationTimeout: new TUint32(fh.add(x.rotationTimeout)).getRaw(),
    registrationFeeAmount: new TUint64(
      fh.add(x.registrationFeeAmount, 0n),
    ).getRaw(),
    dataSizeRange: encRange(fh.add(x.dataSizeRange)),
  };

  // flags should be updated based on input data fields presence
  return { ...data, flags: fh.get() };
}
