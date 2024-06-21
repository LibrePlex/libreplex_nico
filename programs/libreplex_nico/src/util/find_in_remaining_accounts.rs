use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::AccountData;

pub fn find_account<'a>(
    account_id: &Pubkey,
    remaining_accounts: &'a [AccountInfo<'a>],
    name: &str
) -> Option<&'a AccountInfo<'a>> {
    remaining_accounts.iter().find(|x| x.key.eq(account_id))
}

pub fn find_account_data<'a>(
    account_id: &Pubkey,
    account_datas: &'a [AccountData<'a>],
    name: &str
) -> Option<&'a AccountData<'a>> {
    account_datas.iter().find(|x| x.pubkey.eq(account_id))
}

pub fn find_account_or_panic<'a>(
    account_id: &Pubkey,
    remaining_accounts: &'a [AccountInfo<'a>],
    name: &str
) -> &'a AccountInfo<'a> {
    if let Some(x) = find_account(account_id, remaining_accounts, name) {
        x
    } else {
        panic!(
            "Account {} ({}) not found in remaining accounts",
            account_id, name
        );
    }
}
