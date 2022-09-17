use crate::account::Account;
use crate::permissions::BitcoinPermissions;

/// Creates an `Account` struct with send permissions.
pub fn get_child_with_permissions_to_spend() -> Account{
    let account_with_permissions = Account {
        account_id: 1,
        bitcoin_amount: 0,
        permissions: vec![BitcoinPermissions::Send],
        addresses:  vec![],
        pending_transactions: vec![],
        bitcoin_transfered_from_master: 0,
    };
    account_with_permissions
}

/// Creates an `Account` struct with empty permissions.
pub fn get_child_without_permissions_to_spend() -> Account{
    let account_without_permissions = Account {
        account_id: 1,
        bitcoin_amount: 0,
        permissions: vec![],
        addresses:  vec![],
        pending_transactions: vec![],
        bitcoin_transfered_from_master: 0,
    };
    account_without_permissions
}