pub mod transfer_core;
pub mod transfer_nifty;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use transfer_core::{check_and_transfer_core, TransferCoreParams};
use transfer_nifty::{check_and_transfer_nifty, TransferNiftyParams};

use crate::{find_in_remaining_accounts, Nico, NicoType};

pub struct NicoTransferParams<'a, 'b> {
    pub authority_info: Option<&'a AccountInfo<'a>>,
    pub payer_info: &'a AccountInfo<'a>,
    pub asset_info: &'a AccountInfo<'a>,
    pub recipient_info: &'a AccountInfo<'a>,
    pub group_asset_opt_info: Option<&'a AccountInfo<'a>>,
    pub signer_seeds: &'b [&'b [&'b [u8]]],
}

impl<'a, 'b> NicoTransferParams<'a, 'b> {
    pub fn new(
        nico: &'a Nico<'a>,
        payer_info: &'a AccountInfo<'a>,
        recipient_info: &'a AccountInfo<'a>,
        authority_info: Option<&'a AccountInfo<'a>>,
        signer_seeds: &'b [&'b [&'b [u8]]],
        remaining_accounts: &'a [AccountInfo<'a>],
    ) -> NicoTransferParams<'a, 'b> {
        let group_asset_opt_info = nico
            .group
            .map(|x| find_in_remaining_accounts(&x, remaining_accounts, "group"));

        NicoTransferParams {
            asset_info: &nico.account_info,
            authority_info,
            recipient_info,
            group_asset_opt_info,
            payer_info,
            signer_seeds,
        }
    }
}

impl<'a, 'b> Nico<'a> {
    pub fn transfer(
        &'a self,
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
        match self.nico_type {
            NicoType::Nifty => {
                let nifty_params =
                    TransferNiftyParams::from_nico_transfer_params(&params, remaining_accounts);
                check_and_transfer_nifty(nifty_params)
            }
            NicoType::MxCore => {
                let core_params =
                    TransferCoreParams::from_nico_transfer_params(&params, remaining_accounts);
                check_and_transfer_core(core_params)
            },
            _=> {
                panic!("Nico type not supported yet");
            }
        }
    }
}
