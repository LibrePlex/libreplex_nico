use anchor_lang::Key;
use mpl_core::asset;
use mpl_core::types::UpdateAuthority;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::types::TokenStandard;
use nifty_asset::accounts::Asset;
use solana_program::msg;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::{find_account, find_account_data};

pub enum MetadataType {
    // if not enough accounts are provided at construction time
    Unknown,
    // solana labs token-2022 metadata extension
    Token22Extension,
    // non-programmable (tokenStandard: 0)
    MxNonProgrammable,
    // programmable (tokenStandard: 4)
    Mxprogrammable,
}

pub enum NicoType<'a> {
    Nifty,
    MxCore,
    Mint {
        metadata: MetadataType,
        current_owner: Option<&'a AccountInfo<'a>>,
        current_token_account: Option<&'a AccountInfo<'a>>,
    },
}

pub struct Nico<'f> {
    pub nico_type: NicoType<'f>,
    pub pubkey: Pubkey,
    pub owner_program: Pubkey,
    // pub account_info: &'f AccountInfo<'f>,
    pub group: Option<Pubkey>,
}

pub struct AccountData<'f> {
    pub pubkey: Pubkey,
    pub data: &'f [u8]
}

impl<'f> Nico<'f> {


    pub fn from_raw_data(
        pubkey: Pubkey,
        asset_owner_program: Pubkey,
        data: &[u8],
        current_owner: Option<&'f AccountInfo<'f>>,
        current_token_account: Option<&'f AccountInfo<'f>>,
        account_datas: &'f [AccountData<'f>],
    ) -> Nico<'f> {
        
        if asset_owner_program.eq(&nifty_asset::ID) {
            let nifty_asset = Asset::from_bytes(&data).unwrap();
            Nico {
                nico_type: NicoType::Nifty,
                group: nifty_asset.group.to_option(),
                pubkey,
                owner_program: asset_owner_program
            }
        } else if asset_owner_program.eq(&mpl_core::ID) {
            let core_asset = mpl_core::Asset::from_bytes(&data).unwrap();
            msg!("{:?}", core_asset.base.update_authority);
            Nico {
                nico_type: NicoType::MxCore,
                group: match core_asset.base.update_authority {
                    UpdateAuthority::None => None,
                    UpdateAuthority::Address(_) => None,
                    UpdateAuthority::Collection(x) => Some(x),
                },
                pubkey,
                owner_program: mpl_core::ID
            }
        } else if asset_owner_program.eq(&spl_token_2022::ID)
            || asset_owner_program.eq(&spl_token::ID)
        {
            // let's see if metadata exists

            let metadata_pubkey = Pubkey::find_program_address(
                &[
                    "metadata".as_bytes(),
                    &mpl_token_metadata::ID.as_ref(),
                    pubkey.as_ref(),
                ],
                &mpl_token_metadata::ID,
            )
            .0;

            let metadata_account_info_option =
                find_account_data(&metadata_pubkey, account_datas, "mx_metadata");

            if let Some(metadata_account_info) = metadata_account_info_option {
                // ok we have a metadata account in the context.
                // try and deserialize
                let metadata_obj = Metadata::safe_deserialize(metadata_account_info.data).ok();
                if let Some(m) = metadata_obj {
                    return Nico {
                        nico_type: match m.token_standard {
                            Some(TokenStandard::ProgrammableNonFungible) => NicoType::Mint {
                                metadata: MetadataType::Mxprogrammable,
                                current_owner,
                                current_token_account,
                            },
                            Some(TokenStandard::NonFungible) => NicoType::Mint {
                                metadata: MetadataType::MxNonProgrammable,
                                current_owner,
                                current_token_account,
                            },
                            None => NicoType::Mint {
                                metadata: MetadataType::Unknown,
                                current_owner,
                                current_token_account,
                            },
                            _ => {
                                panic!("Unsupported Mx token standard");
                            },
                        },
                        pubkey,
                        group: match m.collection {
                            Some(x) => {
                                if x.verified {
                                    Some(x.key)
                                } else {
                                    None
                                }
                            }
                            None => None,
                        },
                        owner_program: asset_owner_program
                    };
                };
            }
            panic!("No metadata account provided in remaining accounts");
        } else {
            panic!("Unexpected account owner")
        }
    }

    pub fn from_account_info(
        asset_account_info: &'f AccountInfo<'f>,
        current_owner: Option<&'f AccountInfo<'f>>,
        current_token_account: Option<&'f AccountInfo<'f>>,
        remaining_accounts: &'f [AccountInfo<'f>],
    ) -> Nico<'f> {
        let d: &[u8] = &(*asset_account_info.data).borrow();
        
        if asset_account_info.owner.eq(&nifty_asset::ID) {
            let nifty_asset = Asset::from_bytes(&d).unwrap();
            Nico {
                nico_type: NicoType::Nifty,
                group: nifty_asset.group.to_option(),
                pubkey: *asset_account_info.key,
                owner_program: *asset_account_info.owner
            }
        } else if asset_account_info.owner.eq(&mpl_core::ID) {
            let core_asset = mpl_core::Asset::try_from(asset_account_info).unwrap();
            msg!("{:?}", core_asset.base.update_authority);
            Nico {
                nico_type: NicoType::MxCore,
                group: match core_asset.base.update_authority {
                    UpdateAuthority::None => None,
                    UpdateAuthority::Address(_) => None,
                    UpdateAuthority::Collection(x) => Some(x),
                },
                pubkey: *asset_account_info.key,
                owner_program: *asset_account_info.owner
            }
        } else if asset_account_info.owner.eq(&spl_token_2022::ID)
            || asset_account_info.owner.eq(&spl_token::ID)
        {
            // let's see if metadata exists

            let metadata_pubkey = Pubkey::find_program_address(
                &[
                    "metadata".as_bytes(),
                    &mpl_token_metadata::ID.as_ref(),
                    asset_account_info.key.as_ref(),
                ],
                &mpl_token_metadata::ID,
            )
            .0;

            let metadata_account_info_option =
                find_account(&metadata_pubkey, remaining_accounts, "mx_metadata");

            if let Some(metadata_account_info) = metadata_account_info_option {
                // ok we have a metadata account in the context.
                // try and deserialize
                let mut data: &[u8] = &metadata_account_info.try_borrow_data().unwrap()[..];
                let metadata_obj = Metadata::safe_deserialize(&mut data).ok();
                if let Some(m) = metadata_obj {
                    return Nico {
                        nico_type: match m.token_standard {
                            Some(TokenStandard::ProgrammableNonFungible) => NicoType::Mint {
                                metadata: MetadataType::Mxprogrammable,
                                current_owner,
                                current_token_account,
                            },
                            Some(TokenStandard::NonFungible) => NicoType::Mint {
                                metadata: MetadataType::MxNonProgrammable,
                                current_owner,
                                current_token_account,
                            },
                            None => NicoType::Mint {
                                metadata: MetadataType::Unknown,
                                current_owner,
                                current_token_account,
                            },
                            _ => {
                                panic!("Unsupported Mx token standard");
                            },
                        },
                        pubkey: asset_account_info.key(),
                        group: match m.collection {
                            Some(x) => {
                                if x.verified {
                                    Some(x.key)
                                } else {
                                    None
                                }
                            }
                            None => None,
                        },
                        owner_program: *asset_account_info.owner
                    };
                };
            }
            panic!("No metadata account provided in remaining accounts");
        } else {
            panic!("Unexpected account owner")
        }
    }
}
