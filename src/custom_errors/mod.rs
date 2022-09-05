use std::fmt::{Debug, Display};
use std::io;

use std::{fmt, env};



#[derive(Debug)]
pub enum AccountError {
    AccountDoesNotExist(&'static str),
    InsufficientAccount,
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

#[derive(Debug)]
pub enum WalletError {
    SyncElectrumError,
    AddressError,
    BroadcastTransactionError,
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



