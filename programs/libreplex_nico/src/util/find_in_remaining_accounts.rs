
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};


pub fn find_in_remaining_accounts<'a>(
    account_id: &Pubkey,
    remaining_accounts: &'a [AccountInfo<'a>],
    name: &str
) -> &'a AccountInfo<'a> {
    if let Some(x) = remaining_accounts.iter().find(|x| x.key.eq(account_id)) {
        x
    } else {
        panic!("Account {} ({}) not found in remaining accounts", account_id, name);
    }
}
