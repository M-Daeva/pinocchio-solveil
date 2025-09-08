import { PublicKey } from "@solana/web3.js";
import { N } from ".";

export interface InitArgs {
  rotationTimeout?: N<32>;
  accountRegistrationFee?: AssetItem;
  accountDataSizeRange?: Range;
}

export interface AssetItem {
  amount: N<64>;
  asset: PublicKey;
}

export interface Range {
  min: N<32>;
  max: N<32>;
}

export interface UpdateConfigArgs {
  admin?: PublicKey;
  is_paused?: boolean;
  rotation_timeout?: N<32>;
  registration_fee_amount?: N<64>;
  data_size_range?: Range;
}

export interface WithdrawRevenueArgs {
  amount?: N<64>;
}

export interface ReopenAccountArgs {
  maxDataSize: N<32>;
}

export interface ActivateAccountArgs {
  user?: PublicKey;
}

export interface RequestAccountRotationArgs {
  newOwner: PublicKey;
}
