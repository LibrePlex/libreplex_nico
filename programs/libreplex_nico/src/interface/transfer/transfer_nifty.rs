use nifty_asset::{
    instructions::TransferCpi as NiftyTransferCpi, types::Standard as NiftyStandard,
};
use nifty_asset_types::state::Asset;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    assertions::assert_same_pubkeys, find_account_or_panic, find_in_remaining_accounts, Error,
    NicoTransferParams,
};

pub struct TransferNiftyParams<'a, 'b> {
    pub nifty_program_info: &'a AccountInfo<'a>,
    pub signer_info: &'a AccountInfo<'a>,
    pub asset_info: &'a AccountInfo<'a>,
    pub recipient_info: &'a AccountInfo<'a>,
    pub group_asset_opt_info: Option<&'a AccountInfo<'a>>,
    pub signer_seeds: &'b [&'b [&'b [u8]]],
}

impl<'a, 'b, 'c> TransferNiftyParams<'a, 'b> {
    pub fn from_nico_transfer_params(
        params: &NicoTransferParams<'a, 'b>,
        remaining_accounts: &'a [AccountInfo<'a>],
    ) -> TransferNiftyParams<'a, 'b> {
        let nifty_program_info =
            find_account_or_panic(&nifty_asset::ID, remaining_accounts, "nifty_asset");
        TransferNiftyParams {
            nifty_program_info,
            signer_info: match params.authority_info {
                Some(x) => x,
                _ => params.payer_info,
            },
            asset_info: params.asset_info,
            recipient_info: params.recipient_info,
            group_asset_opt_info: params.group_asset_opt_info,
            signer_seeds: params.signer_seeds,
        }
    }
}

pub fn check_and_transfer_nifty(params: TransferNiftyParams<'_, '_>) -> ProgramResult {
    let TransferNiftyParams {
        nifty_program_info,
        signer_info,
        asset_info,
        recipient_info,
        group_asset_opt_info,
        signer_seeds,
    } = params;

    // The incoming asset program is actually the Nifty program.
    assert_same_pubkeys(
        "incoming_asset_program",
        nifty_program_info,
        &nifty_asset::ID,
    )?;

    let data = asset_info.try_borrow_data().unwrap();

    // Must have the expected amount of data and the correct discriminator and standard.
    if data.len() < Asset::LEN || data[2] != NiftyStandard::NonFungible as u8 {
        return Err(Error::InvalidNiftyAsset.into());
    }

    // Drop the data reference before the CPI.
    drop(data);

    // Transfer Nifty asset from authority signer to the swap marker.
    NiftyTransferCpi {
        __program: nifty_program_info,
        asset: asset_info,
        signer: signer_info,
        recipient: recipient_info,
        group: group_asset_opt_info,
    }
    .invoke_signed(signer_seeds)?;
    Ok(())
}
