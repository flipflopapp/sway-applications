contract;

dep abi;
dep errors;

use abi::EnglishAuction;
use errors::{AccessError, InitError, InputError, UserError};

use std::{
    address::Address,
    assert::require,
    block::height,
    chain::auth::{AuthError, msg_sender},
    constants::NATIVE_ASSET_ID,
    context::{call_frames::{contract_id, msg_asset_id}, msg_amount},
    contract_id::ContractId,
    identity::Identity,
    option::Option,
    result::*,
    revert::revert,
    storage::StorageMap,
    token::{force_transfer_to_contract, transfer_to_output}
};

storage {
    buy_asset: ContractId,
    current_bid: u64,
    current_bidder: Identity,
    deposits: StorageMap<Identity, u64>,
    inital_price: u64,
    buyer_withdrawn: bool,
    reserve_price: u64,
    sell_amount: u64,
    sell_asset: ContractId,
    seller: Identity,
    seller_withdawn: bool,
    state: u64,
    end_time: u64,
}

impl EnglishAuction for Contract {

    /// Returns the block at which the auction will end
    ///
    /// # Panics
    ///
    /// The function will panic when:
    /// - The auction has not yet been initalized
    fn auction_end_block() -> u64 {
        require(storage.state != 0, AccessError::AuctionIsNotOpen);
        storage.end_time
    }

    /// Places a bid 
    ///
    /// # Panics
    ///
    /// This function will panic when:
    /// - The auction is not in the bidding state
    /// - The auction is not open
    /// - The asset provided is not the buy asset
    /// - The asset amount provided is less than the inital price if there are no bids
    /// - The bidder is the seller
    /// - The asset amount provided plus current deposit is less than or equal to the current bid
    fn bid() -> bool {
        require(storage.state == 1, AccessError::AuctionIsNotOpen);
        require(height() <= storage.end_time, AccessError::AuctionIsNotOpen);
        require(msg_asset_id() == storage.buy_asset, InputError::IncorrectAssetProvided);

        if (storage.current_bid == 0) {
            require(msg_amount() >= storage.inital_price, InputError::InitalPriceNotMet);
        }

        let sender: Identity = unwrap_identity(msg_sender());
        let balance = storage.deposits.get(sender);
        
        require(!compare_identities(sender, storage.seller), UserError::BidderIsSeller);
        require(msg_amount() + balance >= storage.current_bid, InputError::IncorrectAmountProvided);

        if (msg_amount() + balance < storage.reserve_price) {
            // If the reserve price has not yet been met
            storage.current_bidder = sender;
            storage.current_bid = balance + msg_amount();
            storage.deposits.insert(sender, balance + msg_amount());
        } else {
            // The reserve price was met
            reserve_met(sender, balance);
        }
        true
    }

    /// Purchases at the reserve price
    ///
    /// # Panics
    /// 
    /// This function will panic when:
    /// - The auction is not in the bidding state
    /// - The auction is not open
    /// - There is no reserve price set
    /// - The bidder is the seller
    /// - The asset amount does not meet the reserve price
    /// - The buy assest provided is the incorrect asset
    fn buy_reserve() -> bool {
        require(storage.state == 1, AccessError::AuctionIsNotOpen);
        require(height() <= storage.end_time, AccessError::AuctionIsNotOpen);
        require(storage.reserve_price != 0, AccessError::NoReserveSet);

        let sender: Identity = unwrap_identity(msg_sender());
        let balance = storage.deposits.get(sender);

        require(!compare_identities(sender, storage.seller), UserError::BidderIsSeller);
        require(msg_amount() + balance >= storage.reserve_price, InputError::IncorrectAmountProvided);
        require(msg_asset_id() == storage.buy_asset, InputError::IncorrectAssetProvided);

        reserve_met(sender, balance);
        true
    }

    /// Initalizes the auction with the seller, selling asset, buying asset, 
    /// prices, and length of the auction
    ///
    /// # Panics
    ///
    /// The function will panic when:
    /// - The auction has already been initalized
    /// - The transaction did not have any sell asset
    /// - The transaction's asset is not valid
    /// - The specified buy asset is the 0 address
    /// - The inital price is higher than the reserve price if a reserve price is set
    /// - The time for the auction to end is 0
    fn constructor(seller: Identity, buy_asset: ContractId, inital_price: u64, reserve_price: u64, time: u64) -> bool {
        require(storage.state == 0, InitError::CannotReinitialize);
        require(msg_amount() > 0, InputError::IncorrectAmountProvided);
        require(msg_asset_id() != ~ContractId::from(NATIVE_ASSET_ID), InputError::IncorrectAssetProvided);
        require(buy_asset != ~ContractId::from(NATIVE_ASSET_ID), InitError::BuyAssetNotProvided);
        require((reserve_price >= inital_price && reserve_price != 0) || reserve_price == 0, InitError::ReserveLessThanInitalPrice);
        require(time != 0, InitError::AuctionTimeNotProvided);

        storage.buy_asset = buy_asset;
        storage.buyer_withdrawn = false;
        storage.end_time = time + height();
        storage.inital_price = inital_price;
        storage.reserve_price = reserve_price;
        storage.sell_amount = msg_amount();
        storage.sell_asset = msg_asset_id();
        storage.seller = seller;
        storage.state = 1;
        storage.seller_withdawn = false;

        true
    }

    /// Returns the current bid of the auction
    ///
    /// # Panics
    ///
    /// The function will panic when:
    /// - The auction has not yet been initalized
    fn current_bid() -> u64 {
        require(storage.state != 0, AccessError::AuctionIsNotOpen);
        storage.current_bid
    }

    /// Returns the balance of the Address's buy asset deposits
    ///
    /// # Panics
    ///
    /// The function will panic when:
    /// - The auction has not yet been initalized
    fn deposits(identity: Identity) -> u64 {
        require(storage.state != 0, AccessError::AuctionIsNotOpen);
        storage.deposits.get(identity)
    }

    // Uncomment when https://github.com/FuelLabs/fuels-rs/issues/375 is resolved
    /// Returns the current bidder of the auction
    ///
    /// # Panics
    ///
    /// The function will panic when:
    /// - The auction has not yet been initalized
    // fn highest_bidder() -> Option<Identity> {
    //     require(storage.state != 0, AccessError::AuctionIsNotOpen);
    //     Option::Some(storage.current_bidder)
    // }

    /// Returns the reserve price of the auction
    ///
    /// # Panics
    ///
    /// The function will panic when:
    /// - The auction has not yet been initalized
    fn reserve() -> u64 {
        require(storage.state != 0, AccessError::AuctionIsNotOpen);
        storage.reserve_price
    }

    /// Returns the amount of asset that is being sold
    ///
    /// # Panics
    ///
    /// The function will panic when:
    /// - The auction has not yet been initalized
    fn sell_amount() -> u64 {
        require(storage.state != 0, AccessError::AuctionIsNotOpen);
        storage.sell_amount
    }

    /// Returns the contract id of asset that is being sold
    ///
    /// # Panics
    ///
    /// The function will panic when:
    /// - The auction has not yet been initalized
    fn sell_asset() -> ContractId {
        require(storage.state != 0, AccessError::AuctionIsNotOpen);
        storage.sell_asset
    }

    /// Returns the current state of the function
    fn state() -> u64 {
        storage.state
    }

    /// Withdraws after the end of the auction
    ///
    /// # Panics
    /// 
    /// The function will panic when:
    /// - The auction time is not over
    /// - The auction state is not over
    /// - The buyer is the sender and already withdrew
    /// - The seller is the sender and already withdrew
    /// - The sender is not the buyer or seller and has nothing to withdraw
    fn withdraw() -> bool {
        require(storage.state == 2 || height() >= storage.end_time, AccessError::AuctionIsNotClosed);

        // If time has run out set the contract state to 2
        if (height() >= storage.end_time && storage.state == 1)
        {
            storage.state = 2;
        }

        let sender: Identity = unwrap_identity(msg_sender());
        let current_bidder: Identity = storage.current_bidder;
        let seller: Identity = storage.seller;
            
        if (compare_identities(current_bidder, sender)) {
            // The buyer is withdrawing
            require(!storage.buyer_withdrawn, UserError::UserHasAlreadyWithdrawn);
            storage.buyer_withdrawn = true;
            storage.deposits.insert(sender, 0);

            match sender {
                Identity::Address(sender) => {
                    transfer_to_output(storage.sell_amount, storage.sell_asset, sender);    
                },
                Identity::ContractId(sender) => {
                    force_transfer_to_contract(storage.sell_amount, storage.sell_asset, sender);
                },
            };
        } else if (compare_identities(seller, sender)) {
            // The seller is withdrawing
            require(!storage.seller_withdawn, UserError::UserHasAlreadyWithdrawn);
            storage.seller_withdawn = true;

            // No one placed a bid
            if (storage.current_bid == 0) {
                match sender {
                    Identity::Address(sender) => {
                        transfer_to_output(storage.sell_amount, storage.sell_asset, sender);    
                    },
                    Identity::ContractId(sender) => {
                        force_transfer_to_contract(storage.sell_amount, storage.sell_asset, sender);
                    },
                };
            } else { 
                match sender {
                    Identity::Address(sender) => {
                        transfer_to_output(storage.current_bid, storage.buy_asset, sender);    
                    },
                    Identity::ContractId(sender) => {
                        force_transfer_to_contract(storage.current_bid, storage.buy_asset, sender);
                    },
                };
            }
        } else {
            // Anyone with a failed bid is withdrawing
            let deposit_amount = storage.deposits.get(sender);
            require(deposit_amount > 0, UserError::UserHasAlreadyWithdrawn);

            storage.deposits.insert(sender, 0);

            match sender {
                Identity::Address(sender) => {
                    transfer_to_output(deposit_amount, storage.buy_asset, sender);    
                },
                Identity::ContractId(sender) => {
                    force_transfer_to_contract(deposit_amount, storage.buy_asset, sender);
                },
            };
        };
        true
    }
}

// This function will take two identities and return true if they are the same
fn compare_identities(identity1: Identity, identity2: Identity) -> bool {
    match identity1 {
        Identity::Address(identity1) => {
            match identity2 {
                Identity::Address(identity2) => identity1.value == identity2.value,
                _ => false,
            }
        },
        Identity::ContractId(identity1) => {
            match identity2 {
                Identity::ContractId(identity2) => identity1.value == identity2.value,
                _ => false,
            }
        }
    }
}

// Gets called when the reserve price is met
fn reserve_met(sender: Identity, balance: u64) {
    storage.state = 2;
    storage.current_bidder = sender;
    storage.current_bid = storage.reserve_price;
    storage.buyer_withdrawn = true;
    storage.deposits.insert(sender, 0);

    match sender {
        Identity::Address(sender) => {
            transfer_to_output(storage.sell_amount, storage.sell_asset, sender);    
        },
        Identity::ContractId(sender) => {
            force_transfer_to_contract(storage.sell_amount, storage.sell_asset, sender);
        },
    };

    let overpaid_balance: u64 = (msg_amount() + balance) - storage.reserve_price;
    if (overpaid_balance > 0)
    {
        match sender {
            Identity::Address(sender) => {
                transfer_to_output(overpaid_balance, storage.buy_asset, sender);    
            },
            Identity::ContractId(sender) => {
                force_transfer_to_contract(overpaid_balance, storage.buy_asset, sender);
            },
        };
    }
}

fn unwrap_identity(sender: Result<Identity, AuthError>) -> Identity {
    sender.unwrap()
}
