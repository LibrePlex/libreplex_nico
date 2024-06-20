use std::str::FromStr;

use anchor_lang::AnchorDeserialize;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use libreplex_shared::sysvar_instructions_program;
use mpl_token_metadata::{
    accounts::Metadata,
    instructions::{TransferV1Cpi, TransferV1InstructionArgs},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey, system_program,
};

use crate::{
    assertions::assert_same_pubkeys, find_account_or_panic, Error,
    NicoTransferParams,
};

pub const AUTH_RULES: &str = "AdH2Utn6Fus15ZhtenW4hZBQnvtLgM1YCW2MfVp7pYS5";

pub struct TransferPnftParams<'a, 'b> {
    pub mpl_token_program_info: &'a AccountInfo<'a>,
    pub authority_info: Option<&'a AccountInfo<'a>>,
    pub asset_info: &'a AccountInfo<'a>,
    pub new_owner_info: &'a AccountInfo<'a>,
    pub payer_info: &'a AccountInfo<'a>,
    pub system_program_info: &'a AccountInfo<'a>,
    // pub collection_asset_opt_info: Option<&'a AccountInfo<'a>>,
    source_token_account_info: &'a AccountInfo<'a>,
    current_owner: &'a AccountInfo<'a>,
    target_token_account_info: &'a AccountInfo<'a>,
    metadata: &'a AccountInfo<'a>,
    edition: &'a AccountInfo<'a>,
    source_token_record_info: &'a AccountInfo<'a>,
    target_token_record_info: &'a AccountInfo<'a>,
    sysvar_instruction_info: &'a AccountInfo<'a>,
    spl_token_program_info: &'a AccountInfo<'a>,
    spl_ata_program: &'a AccountInfo<'a>,
    auth_rules_program_info: &'a AccountInfo<'a>,
    auth_rules_info: Option<&'a AccountInfo<'a>>,
    pub signer_seeds: &'b [&'b [&'b [u8]]],
}

impl<'a, 'b, 'c> TransferPnftParams<'a, 'b> {
    pub fn from_nico_transfer_params(
        current_owner: &'a AccountInfo<'a>,
        current_token_account: &'a AccountInfo<'a>,
        params: &NicoTransferParams<'a, 'b>,
        // need source token account as it's the only one
        // that cannot be derived if it's not an ATA
        // target token accounts are forced to use ATA
        remaining_accounts: &'a [AccountInfo<'a>],
    ) -> TransferPnftParams<'a, 'b> {
        // need to derive extra system account

        let system_program_info =
            find_account_or_panic(&system_program::ID, remaining_accounts, "system_program");

        let mpl_token_program_info = find_account_or_panic(
            &mpl_token_metadata::ID,
            remaining_accounts,
            "mpl_core_program",
        );

        let token_program = find_account_or_panic(
            &params.asset_info.owner,
            remaining_accounts,
            "token_program",
        );
        let spl_ata_program = find_account_or_panic(
            &spl_associated_token_account::ID,
            remaining_accounts,
            "associated_token_program",
        );
        let auth_rules_program_info = find_account_or_panic(
            &Pubkey::from_str(AUTH_RULES).unwrap(),
            remaining_accounts,
            "auth_rules_program",
        );

        let metadata_info = find_account_or_panic(
            &Pubkey::find_program_address(
                &[
                    "metadata".as_bytes(),
                    &mpl_token_metadata::ID.as_ref(),
                    params.asset_info.key.as_ref(),
                ],
                &mpl_token_metadata::ID,
            )
            .0,
            remaining_accounts,
            "metadata",
        );

        let mut bytes: &[u8] = &(*metadata_info.data).borrow();
        let metadata = Metadata::deserialize(&mut bytes)
            .map_err(|error| {
                msg!("Error: {}", error);
                Error::DeserializationError
            })
            .unwrap();

        let auth_rules_info = match metadata.programmable_config {
            Some(x) => match &x {
                mpl_token_metadata::types::ProgrammableConfig::V1 { rule_set } => *rule_set,
            },
            None => None,
        }
        .map(|x| find_account_or_panic(&x, remaining_accounts, "auth_rule"));

        let sysvar_instruction_info = find_account_or_panic(
            &sysvar_instructions_program::ID,
            remaining_accounts,
            "sysvar_instructions_program",
        );

        let target_token_account_info = find_account_or_panic(
            &get_associated_token_address_with_program_id(
                &params.recipient_info.key,
                &params.asset_info.key,
                &params.asset_info.owner,
            ),
            remaining_accounts,
            "target_ata",
        );

        let source_token_record_info = find_account_or_panic(
            &Pubkey::find_program_address(
                &[
                    b"metadata",
                    mpl_token_metadata::ID.as_ref(),
                    params.asset_info.key.as_ref(),
                    b"token_record",
                    current_token_account.key.as_ref(),
                ],
                &mpl_token_metadata::ID,
            )
            .0,
            remaining_accounts,
            "source_token_record",
        );

        let target_token_record_info = find_account_or_panic(
            &Pubkey::find_program_address(
                &[
                    b"metadata",
                    mpl_token_metadata::ID.as_ref(),
                    params.asset_info.key.as_ref(),
                    b"token_record",
                    target_token_account_info.key.as_ref(),
                ],
                &mpl_token_metadata::ID,
            )
            .0,
            remaining_accounts,
            "target_token_record",
        );

        let edition_info = find_account_or_panic(
            &Pubkey::find_program_address(
                &[
                    "metadata".as_bytes(),
                    &mpl_token_metadata::ID.as_ref(),
                    params.asset_info.key.as_ref(),
                    "edition".as_bytes(),
                ],
                &mpl_token_metadata::ID,
            )
            .0,
            remaining_accounts,
            "master_edition",
        );

        TransferPnftParams {
            mpl_token_program_info,
            authority_info: params.authority_info,
            asset_info: params.asset_info,
            new_owner_info: params.recipient_info,
            // collection_asset_opt_info: params.group_asset_opt_info,
            signer_seeds: params.signer_seeds,
            payer_info: params.payer_info,
            system_program_info,
            source_token_account_info: current_token_account,
            current_owner,
            target_token_account_info,
            metadata: metadata_info,
            edition: edition_info,
            source_token_record_info,
            target_token_record_info,
            sysvar_instruction_info,
            spl_token_program_info: token_program,
            spl_ata_program,
            auth_rules_program_info,
            auth_rules_info,
        }
    }
}

pub fn check_and_transfer_pnft(params: TransferPnftParams<'_, '_>) -> ProgramResult {
    let TransferPnftParams {
        source_token_account_info,
        current_owner,
        target_token_account_info,
        metadata,
        edition,
        source_token_record_info,
        target_token_record_info,
        sysvar_instruction_info,
        spl_token_program_info,
        spl_ata_program,
        auth_rules_program_info,
        auth_rules_info,
        mpl_token_program_info,
        authority_info,
        payer_info,
        asset_info,
        new_owner_info,
        signer_seeds,
        system_program_info,
    } = params;

    // The incoming asset program is actually the Nifty program.
    assert_same_pubkeys(
        "incoming_asset_program",
        mpl_token_program_info,
        &mpl_token_metadata::ID,
    )?;

    assert_same_pubkeys("system_program", system_program_info, &system_program::ID)?;

    let data = asset_info.try_borrow_data().unwrap();

    // Drop the data reference before the CPI.
    drop(data);
    TransferV1Cpi {
        __program: mpl_token_program_info,
        token: source_token_account_info,
        token_owner: current_owner,
        destination_token: target_token_account_info,
        destination_owner: new_owner_info,
        mint: asset_info,
        metadata,
        edition: Some(edition),
        token_record: Some(source_token_record_info),
        destination_token_record: Some(target_token_record_info),
        authority: authority_info.map_or(payer_info, |x| x),
        payer: payer_info,
        system_program: system_program_info,
        sysvar_instructions: sysvar_instruction_info,
        spl_token_program: spl_token_program_info,
        spl_ata_program,
        authorization_rules_program: Some(auth_rules_program_info),
        authorization_rules: auth_rules_info,
        __args: {
            TransferV1InstructionArgs {
                amount: 1,
                authorization_data: None,
            }
        },
    }
    .invoke_signed(signer_seeds)?;

    Ok(())
}
