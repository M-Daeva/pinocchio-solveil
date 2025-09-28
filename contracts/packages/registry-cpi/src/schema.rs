use {
    crate::{
        state::{Bump, Config, RotationState, UserAccount, UserCounter, UserId},
        types::common::{AssetItem, Range, StructWithEnum, TargetEnum},
    },
    base::types::{
        BitField, String16, String32, String4096, String64, Uint128, Uint16, Uint32, Uint64,
    },
};

// Add a function to collect all TypeScript
pub fn get_all_typescript_interfaces() -> String {
    let mut output = String::new();

    for item in [
        BitField::typescript_interface(),
        Uint16::typescript_interface(),
        Uint32::typescript_interface(),
        Uint64::typescript_interface(),
        Uint128::typescript_interface(),
        String16::typescript_interface(),
        String32::typescript_interface(),
        String64::typescript_interface(),
        String4096::typescript_interface(),
        //
        AssetItem::typescript_interface(),
        Range::typescript_interface(),
        TargetEnum::typescript_interface(),
        StructWithEnum::typescript_interface(),
        //
        Bump::typescript_interface(),
        Config::typescript_interface(),
        UserCounter::typescript_interface(),
        RotationState::typescript_interface(),
        UserId::typescript_interface(),
        UserAccount::typescript_interface(),
    ] {
        output.push_str(item);
    }

    output
}
