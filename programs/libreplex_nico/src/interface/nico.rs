use anchor_lang::Key;
use mpl_core::types::UpdateAuthority;
use nifty_asset::accounts::Asset;
use solana_program::msg;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

pub enum NicoType {
    Nifty,
    Core,
}

pub struct Nico<'f> {
    pub nico_type: NicoType,
    pub pubkey: Pubkey,
    pub account_info: AccountInfo<'f>,
    pub group: Option<Pubkey>,
}

impl<'f, 'g: 'f> Nico<'f> {
    pub fn from_account_info(account_info: AccountInfo<'f>) -> Nico<'f> {
        let d = account_info.try_borrow_data().unwrap();

        if account_info.owner.eq(&mpl_core::ID) {
            let core_asset = mpl_core::Asset::try_from(&account_info).unwrap();
            msg!("{:?}", core_asset.base.update_authority);
            Nico {
                nico_type: NicoType::Core,
                group: match core_asset.base.update_authority {
                    UpdateAuthority::None => None,
                    UpdateAuthority::Address(_) => None,
                    UpdateAuthority::Collection(x) => Some(x),
                },
                pubkey: account_info.key(),
                account_info: account_info.clone(),
            }
        } else if account_info.owner.eq(&nifty_asset::ID) {
            let nifty_asset = Asset::from_bytes(&d).unwrap();
            Nico {
                nico_type: NicoType::Nifty,
                group: nifty_asset.group.to_option(),
                pubkey: account_info.key(),
                account_info: account_info.clone(),
            }
        } else {
            panic!("Unexpected account owner")
        }
    }

    
}
