contract;

dep interface;
dep data_structures;
dep errors;
dep events;

use std::{
    chain::auth::msg_sender,
    constants::BASE_ASSET_ID,
    identity::Identity,
    logging::log,
    option::Option,
    result::Result,
    revert::require,
};

use interface::Oracle;
use data_structures::State;
use errors::{AccessError, InitializationError};
use events::PriceUpdateEvent;

storage {
    /// The Identity that can control the oracle (node)
    owner: Option<Identity> = Option::None,
    /// Current price of tracked asset
    price: u64 = 0,
    /// The initialization state of the contract.
    state: State = State::NotInitialized,
}

impl Oracle for Contract {
    #[storage(read, write)] fn constructor(owner: Identity) {
        require(storage.state == State::NotInitialized, InitializationError::CannotReinitialize);

        storage.owner = Option::Some(owner);
        storage.state = State::Initialized;
    }

    #[storage(read)] fn owner() -> Option<Identity> {
        storage.owner
    }

    #[storage(read)] fn price() -> u64 {
        storage.price
    }

    #[storage(read, write)] fn set_price(price: u64) {
        require(storage.state == State::Initialized, InitializationError::ContractNotInitialized);
        require(msg_sender().unwrap() == storage.owner.unwrap(), AccessError::NotOwner);

        storage.price = price;

        log(PriceUpdateEvent {
            price
        });
    }
}
