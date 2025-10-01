import { IAssetItem, IRange } from "./types";
import { IBump, IConfig, IUserCounter, IRotationState, IUserId, IUserAccount } from "./accounts";
import { IRequestAccountRotationInstructionDataArgs, IWithdrawRevenueInstructionDataArgs, IUpdateConfigInstructionDataArgs, IActivateAccountInstructionDataArgs, ICreateAccountInstructionDataArgs, IInitInstructionDataArgs, IConfirmAccountRotationInstructionDataArgs, IConfirmAdminRotationInstructionDataArgs, IReopenAccountInstructionDataArgs, ICloseAccountInstructionDataArgs, IWriteDataInstructionDataArgs } from "./instructions";
import { AssetItem, Range, Bump, Config, UserCounter, RotationState, UserId, UserAccount, RequestAccountRotationInstructionDataArgs, WithdrawRevenueInstructionDataArgs, UpdateConfigInstructionDataArgs, ActivateAccountInstructionDataArgs, CreateAccountInstructionDataArgs, InitInstructionDataArgs, ConfirmAccountRotationInstructionDataArgs, ConfirmAdminRotationInstructionDataArgs, ReopenAccountInstructionDataArgs, CloseAccountInstructionDataArgs, WriteDataInstructionDataArgs } from "../../codama";
import { TAddress, TBitField, TBitFieldBuilder, TString4096, TUint32, TUint64, TUint8 } from "../../../interfaces/primitives";

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

export function encBump(x: IBump): Bump {
  return {
    config: new TUint8(x.config).getRaw(),
    userCounter: new TUint8(x.userCounter).getRaw(),
    rotationState: new TUint8(x.rotationState).getRaw(),
  };
}

export function decBump(x: Bump): IBump {
  return {
    config: new TUint8(x.config).get(),
    userCounter: new TUint8(x.userCounter).get(),
    rotationState: new TUint8(x.rotationState).get(),
  };
}

export function encConfig(x: IConfig): Config {
  return {
    flags: new TBitFieldBuilder()
      .withBool(x.isPaused)
      .build().getRaw(),
    admin: new TAddress(x.admin).getRaw(),
    rotationTimeout: new TUint32(x.rotationTimeout).getRaw(),
    registrationFee: encAssetItem(x.registrationFee),
    dataSizeRange: encRange(x.dataSizeRange),
  };
}

export function decConfig(x: Config): IConfig {
  return {
    isPaused: new TBitField(x.flags).get(0),
    admin: new TAddress(x.admin).get(),
    rotationTimeout: new TUint32(x.rotationTimeout).get(),
    registrationFee: decAssetItem(x.registrationFee),
    dataSizeRange: decRange(x.dataSizeRange),
  };
}

export function encUserCounter(x: IUserCounter): UserCounter {
  return {
    lastUserId: new TUint32(x.lastUserId).getRaw(),
  };
}

export function decUserCounter(x: UserCounter): IUserCounter {
  return {
    lastUserId: new TUint32(x.lastUserId).get(),
  };
}

export function encRotationState(x: IRotationState): RotationState {
  return {
    owner: new TAddress(x.owner).getRaw(),
    newOwner: new TAddress(x.newOwner).getRaw(),
    expirationDate: new TUint64(x.expirationDate).getRaw(),
  };
}

export function decRotationState(x: RotationState): IRotationState {
  return {
    owner: new TAddress(x.owner).get(),
    newOwner: new TAddress(x.newOwner).get(),
    expirationDate: new TUint64(x.expirationDate).get(),
  };
}

export function encUserId(x: IUserId): UserId {
  return {
    flags: new TBitFieldBuilder()
      .withBool(x.isOpen)
      .withBool(x.isActivated)
      .build().getRaw(),
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

export function encUserAccount(x: IUserAccount): UserAccount {
  return {
    data: new TString4096(x.data).getRaw(),
    nonce: new TUint64(x.nonce).getRaw(),
    maxSize: new TUint32(x.maxSize).getRaw(),
  };
}

export function decUserAccount(x: UserAccount): IUserAccount {
  return {
    data: new TString4096(x.data).get(),
    nonce: new TUint64(x.nonce).get(),
    maxSize: new TUint32(x.maxSize).get(),
  };
}

export function encRequestAccountRotationInstructionDataArgs(x: IRequestAccountRotationInstructionDataArgs): RequestAccountRotationInstructionDataArgs {
  return {
    newOwner: new TAddress(x.newOwner).getRaw(),
  };
}

export function encWithdrawRevenueInstructionDataArgs(x: IWithdrawRevenueInstructionDataArgs): WithdrawRevenueInstructionDataArgs {
  return {
    flags: new TBitFieldBuilder()
      .withOptNonBool(x.amount)
      .build().getRaw(),
    amount: new TUint64(x.amount).getRaw(),
  };
}

export function encUpdateConfigInstructionDataArgs(x: IUpdateConfigInstructionDataArgs): UpdateConfigInstructionDataArgs {
  return {
    flags: new TBitFieldBuilder()
      .withOptBool(x.isPaused)
      .withOptNonBool(x.admin)
      .withOptNonBool(x.rotationTimeout)
      .withOptNonBool(x.registrationFeeAmount)
      .withOptNonBool(x.dataSizeRange)
      .build().getRaw(),
    admin: new TAddress(x.admin).getRaw(),
    rotationTimeout: new TUint32(x.rotationTimeout).getRaw(),
    registrationFeeAmount: new TUint64(x.registrationFeeAmount).getRaw(),
    dataSizeRange: encRange(x.dataSizeRange),
  };
}

export function encActivateAccountInstructionDataArgs(x: IActivateAccountInstructionDataArgs): ActivateAccountInstructionDataArgs {
  return {
  };
}

export function encCreateAccountInstructionDataArgs(x: ICreateAccountInstructionDataArgs): CreateAccountInstructionDataArgs {
  return {
    maxDataSize: new TUint32(x.maxDataSize).getRaw(),
  };
}

export function encInitInstructionDataArgs(x: IInitInstructionDataArgs): InitInstructionDataArgs {
  return {
    flags: new TBitFieldBuilder()
      .withOptNonBool(x.rotationTimeout)
      .withOptNonBool(x.accountRegistrationFee)
      .withOptNonBool(x.accountDataSizeRange)
      .build().getRaw(),
    rotationTimeout: new TUint32(x.rotationTimeout).getRaw(),
    accountRegistrationFee: encAssetItem(x.accountRegistrationFee),
    accountDataSizeRange: encRange(x.accountDataSizeRange),
  };
}

export function encConfirmAccountRotationInstructionDataArgs(x: IConfirmAccountRotationInstructionDataArgs): ConfirmAccountRotationInstructionDataArgs {
  return {
  };
}

export function encConfirmAdminRotationInstructionDataArgs(x: IConfirmAdminRotationInstructionDataArgs): ConfirmAdminRotationInstructionDataArgs {
  return {
  };
}

export function encReopenAccountInstructionDataArgs(x: IReopenAccountInstructionDataArgs): ReopenAccountInstructionDataArgs {
  return {
    maxDataSize: new TUint32(x.maxDataSize).getRaw(),
  };
}

export function encCloseAccountInstructionDataArgs(x: ICloseAccountInstructionDataArgs): CloseAccountInstructionDataArgs {
  return {
  };
}

export function encWriteDataInstructionDataArgs(x: IWriteDataInstructionDataArgs): WriteDataInstructionDataArgs {
  return {
    data: new TString4096(x.data).getRaw(),
    nonce: new TUint64(x.nonce).getRaw(),
  };
}
