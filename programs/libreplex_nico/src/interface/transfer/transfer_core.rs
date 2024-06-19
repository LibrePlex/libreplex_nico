use mpl_core::instructions::{TransferV1Cpi as MplCoreTransferCpi, TransferV1InstructionArgs};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

use crate::{assertions::assert_same_pubkeys, find_in_remaining_accounts, NicoTransferParams};
pub struct TransferCoreParams<'a, 'b> {
    pub mpl_core_program_info: &'a AccountInfo<'a>,
    pub authority_info: Option<&'a AccountInfo<'a>>,
    pub asset_info: &'a AccountInfo<'a>,
    pub new_owner_info: &'a AccountInfo<'a>,
    pub payer_info: &'a AccountInfo<'a>,
    pub system_program_info: &'a AccountInfo<'a>,
    pub collection_asset_opt_info: Option<&'a AccountInfo<'a>>,
    pub signer_seeds: &'b [&'b [&'b [u8]]],
}

impl<'a, 'b> TransferCoreParams<'a, 'b> {
    pub fn from_nico_transfer_params(
        params: &NicoTransferParams<'a, 'b>,
        remaining_accounts: &'a [AccountInfo<'a>],
    ) -> TransferCoreParams<'a, 'b> {
        // need to derive extra system account
        let system_program_info =
            find_in_remaining_accounts(&system_program::ID, remaining_accounts, "system_program");

        TransferCoreParams {
            mpl_core_program_info: params.asset_owner_program,
            authority_info: params.authority_info,
            asset_info: params.asset_info,
            new_owner_info: params.recipient_info,
            collection_asset_opt_info: params.group_asset_opt_info,
            signer_seeds: params.signer_seeds,
            payer_info: params.payer_info,
            system_program_info,
        }
    }
}

pub fn check_and_transfer_core(params: TransferCoreParams<'_, '_>) -> ProgramResult {
    let TransferCoreParams {
        mpl_core_program_info,
        authority_info,
        payer_info,
        asset_info,
        new_owner_info,
        collection_asset_opt_info,
        signer_seeds,
        system_program_info,
    } = params;

    // The incoming asset program is actually the Nifty program.
    assert_same_pubkeys(
        "incoming_asset_program",
        mpl_core_program_info,
        &mpl_core::ID,
    )?;

    assert_same_pubkeys("system_program", system_program_info, &system_program::ID)?;

    let data = asset_info.try_borrow_data().unwrap();

    // Drop the data reference before the CPI.
    drop(data);

    MplCoreTransferCpi {
        __program: mpl_core_program_info,
        asset: asset_info,
        collection: collection_asset_opt_info,
        payer: payer_info,
        authority: authority_info,
        new_owner: new_owner_info,
        __args: TransferV1InstructionArgs {
            compression_proof: None,
        },
        system_program: None,
        log_wrapper: None,
    }
    .invoke_signed(signer_seeds)?;
    Ok(())
}
