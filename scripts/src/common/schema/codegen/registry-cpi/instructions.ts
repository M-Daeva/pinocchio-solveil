import { Address } from "gill";
import { IRange, IAssetItem } from "./types";

export interface IRequestAccountRotationInstructionDataArgs {
  newOwner: Address;
}

export interface IWithdrawRevenueInstructionDataArgs {
  amount?: bigint;
}

export interface IUpdateConfigInstructionDataArgs {
  isPaused?: boolean;
  admin?: Address;
  rotationTimeout?: number;
  registrationFeeAmount?: bigint;
  dataSizeRange?: IRange;
}

export interface IActivateAccountInstructionDataArgs {
}

export interface ICreateAccountInstructionDataArgs {
  maxDataSize: number;
}

export interface IInitInstructionDataArgs {
  rotationTimeout?: number;
  accountRegistrationFee?: IAssetItem;
  accountDataSizeRange?: IRange;
}

export interface IConfirmAccountRotationInstructionDataArgs {
}

export interface IConfirmAdminRotationInstructionDataArgs {
}

export interface IReopenAccountInstructionDataArgs {
  maxDataSize: number;
}

export interface ICloseAccountInstructionDataArgs {
}

export interface IWriteDataInstructionDataArgs {
  data: string;
  nonce: bigint;
}
