pub mod transfer_core;
pub mod transfer_nifty;
pub mod transfer_pnft;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use transfer_core::{check_and_transfer_core, TransferCoreParams};
use transfer_nifty::{check_and_transfer_nifty, TransferNiftyParams};
use transfer_pnft::{check_and_transfer_pnft, TransferPnftParams};

use crate::{find_account_or_panic, Nico, NicoType};

pub struct NicoTransferParams<'a, 'b> {
    pub nico_pubkey: Pubkey,
    pub nico_owner_program: Pubkey,
    pub authority_info: Option<&'a AccountInfo<'a>>,
    pub payer_info: &'a AccountInfo<'a>,
    // pub asset_info: &'a AccountInfo<'a>,
    pub recipient_info: &'a AccountInfo<'a>,
    pub group_asset_opt_info: Option<&'a AccountInfo<'a>>,
    pub signer_seeds: &'b [&'b [&'b [u8]]],
}

impl<'a: 'c, 'b, 'c> NicoTransferParams<'a, 'b> {
    pub fn new(
        nico: &'c Nico<'a>,
        payer_info: &'a AccountInfo<'a>,
        recipient_info: &'a AccountInfo<'a>,
        authority_info: Option<&'a AccountInfo<'a>>,
        signer_seeds: &'b [&'b [&'b [u8]]],
        remaining_accounts: &'a [AccountInfo<'a>],
    ) -> NicoTransferParams<'a, 'b> {
        let group_asset_opt_info = nico
            .group
            .map(|x| find_account_or_panic(&x, remaining_accounts, "group"));

        NicoTransferParams {
            // asset_info: &nico.account_info,
            nico_owner_program: nico.owner_program,
            nico_pubkey: nico.pubkey,
            authority_info,
            recipient_info,
            group_asset_opt_info,
            payer_info,
            signer_seeds,
        }
    }
}

impl<'a: 'c, 'b, 'c> Nico<'a> {
    pub fn transfer(
        &'c self,
        asset_info: &'a AccountInfo<'a>,
        payer: &'a AccountInfo<'a>,
        target_wallet: &'a AccountInfo<'a>,
        authority: Option<&'a AccountInfo<'a>>,
        remaining_accounts: &'a [AccountInfo<'a>],
        signer_seeds: &'b [&'b [&'b [u8]]],
    ) -> ProgramResult {
        let params = NicoTransferParams::new(
            self,
            payer,
            target_wallet,
            authority,
            signer_seeds,
            remaining_accounts,
        );

        match &self.nico_type {
            NicoType::Nifty => {
                let nifty_params = TransferNiftyParams::from_nico_transfer_params(
                    asset_info,
                    &params,
                    remaining_accounts,
                );
                check_and_transfer_nifty(nifty_params)
            }
            NicoType::MxCore => {
                let core_params = TransferCoreParams::from_nico_transfer_params(
                    asset_info,
                    &params,
                    remaining_accounts,
                );
                check_and_transfer_core(core_params)
            }
            NicoType::Mint {
                metadata,
                current_owner,
                current_token_account,
            } => match &metadata {
                crate::MetadataType::Unknown => todo!(),
                crate::MetadataType::Token22Extension => {
                    panic!("Nico type MxNonProgrammable not supported yet");
                }
                crate::MetadataType::MxNonProgrammable => {
                    panic!("Nico type MxNonProgrammable not supported yet");
                }
                crate::MetadataType::Mxprogrammable => {
                    let programmable_mx_params = TransferPnftParams::from_nico_transfer_params(
                        asset_info,
                        current_owner.unwrap_or_else(||panic!("This Nico was constructed without current owner. Cannot transfer")),
                        current_token_account.unwrap_or_else(||panic!("This Nico was constructed without current token account. Cannot transfer")),
                        &params,
                        remaining_accounts,
                    );
                    check_and_transfer_pnft(programmable_mx_params)
                }
            },
        }
    }
}
