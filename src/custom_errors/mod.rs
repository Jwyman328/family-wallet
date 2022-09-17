use std::fmt::{Debug, Display};
use std::io;

use std::{fmt, env};


/// An Error with an individual `Account`
#[derive(Debug)]
pub enum AccountError {
    /// If searching for an `Account` and it does not exist.
    AccountDoesNotExist(&'static str),
    /// If an action attempted is not permitted on this `Account`.
    InsufficientAccount,
    /// A general catch all `Account` error.
    Default(&'static str),
}


impl Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
        AccountError::AccountDoesNotExist(_e) => write!(f, "Account does not exist"),
        AccountError::Default(_e) => write!(f, "Default account error"),
        AccountError::InsufficientAccount  => write!(f, "Account is insufficient to take desired action"),
      }
    }
  }


// Implement std::convert::From for AppError; from io::Error
impl From<io::Error> for AccountError {
    fn from(_error: io::Error) -> Self {
        
        return AccountError::AccountDoesNotExist("AccountDoesNotExist") 
    }
}

/// An error related to the Wallet functionality.
#[derive(Debug)]
pub enum WalletError {
    /// An error due to not being able to connect with the electrum server.
    SyncElectrumError,
    /// An error related to a bitcoin address, for example searching for an address that does not exist.
    AddressError,
    /// An error associated when trying to broadcast to the bitcoin network.
    BroadcastTransactionError,
    /// An error associated with the wallet's public or private keys.
    /// For example trying to create an xpriv from an invalid mnemonic phrase. 
    KeyError,
}


impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
        WalletError::SyncElectrumError => write!(f, "Error syncing electrum server"),
        WalletError::AddressError => write!(f, "Bitcoin address error"),
        WalletError::BroadcastTransactionError => write!(f, "Error broadcasting transaction"),
        WalletError::KeyError => write!(f, "Key error"),
      }
    }
  }


impl From<env::VarError> for WalletError {
    fn from(_error: env::VarError) -> Self {
        return WalletError::SyncElectrumError
    }
}

impl From<bdk::electrum_client::Error> for WalletError {
    fn from(_error: bdk::electrum_client::Error) -> Self {
        return WalletError::SyncElectrumError
    }
}

impl From<bdk::Error> for WalletError {
    fn from(_error: bdk::Error) -> Self {
        return WalletError::SyncElectrumError
    }
}

impl From<bdk::bitcoin::util::address::Error> for WalletError {
    fn from(_error: bdk::bitcoin::util::address::Error) -> Self {
        return WalletError::AddressError
    }
}

impl From<bdk::keys::bip39::Error> for WalletError {
    fn from(_error: bdk::keys::bip39::Error) -> Self {
        return WalletError::KeyError
    }
}

impl From<bdk::keys::KeyError> for WalletError {
    fn from(_error: bdk::keys::KeyError) -> Self {
        return WalletError::KeyError
    }
}



