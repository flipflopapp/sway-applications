// TODO: execute_transaction, transfer have incomplete tests!

mod utils;

use fuels::signers::Signer;
use sha2::{Digest, Sha256};
use utils::{
    abi_calls::{
        balance, constructor, execute_transaction, nonce, owner, transaction_hash, transfer,
    },
    test_helpers::{deposit, mint, setup},
    Identity, Owner, User, B512,
};

mod balance {

    use super::*;

    mod success {

        use super::*;

        #[tokio::test]
        async fn returns_zero_for_nonexistent_asset() {
            let (multisig, _, asset) = setup().await;
            assert_eq!(0, balance(asset.id, &multisig.contract).await);
        }

        #[tokio::test]
        async fn returns_correct_balance() {
            let (multisig, wallets, asset) = setup().await;

            assert_eq!(0, balance(asset.id, &multisig.contract).await);

            mint(100, &asset.contract, &wallets.users[0]).await;
            deposit(100, asset.id, multisig.id, &wallets.users[0]).await;

            assert_eq!(100, balance(asset.id, &multisig.contract).await);
        }
    }
}

mod constructor {

    use super::*;

    mod success {

        use super::*;

        #[tokio::test]
        async fn initializes() {
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            assert_eq!(0, nonce(&multisig.contract).await);
            assert_eq!(
                Owner { weight: 0 },
                owner(&multisig.contract, &wallets.users[0]).await
            );
            assert_eq!(
                Owner { weight: 0 },
                owner(&multisig.contract, &wallets.users[1]).await
            );
            assert_eq!(
                Owner { weight: 0 },
                owner(&multisig.contract, &wallets.users[2]).await
            );

            constructor(&multisig.contract, 2, users.clone()).await;

            assert_eq!(1, nonce(&multisig.contract).await);
            assert_eq!(
                Owner {
                    weight: users.get(0).unwrap().weight
                },
                owner(&multisig.contract, &wallets.users[0]).await
            );
            assert_eq!(
                Owner {
                    weight: users.get(1).unwrap().weight
                },
                owner(&multisig.contract, &wallets.users[1]).await
            );
            assert_eq!(
                Owner {
                    weight: users.get(2).unwrap().weight
                },
                owner(&multisig.contract, &wallets.users[2]).await
            );
        }
    }

    mod revert {

        use super::*;

        #[tokio::test]
        #[should_panic]
        async fn when_nonce_is_not_zero() {
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            constructor(&multisig.contract, 2, users.clone()).await;
            constructor(&multisig.contract, 2, users).await;
        }

        #[tokio::test]
        #[should_panic]
        async fn when_threshold_is_zero() {
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            constructor(&multisig.contract, 0, users.clone()).await;
        }

        #[tokio::test]
        #[should_panic]
        async fn when_user_weight_is_zero() {
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 0,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            constructor(&multisig.contract, 2, users.clone()).await;
        }
    }
}

mod execute_transaction {

    use super::*;

    mod success {

        use super::*;

        #[tokio::test]
        #[ignore]
        async fn executes() {
            // TODO: add call to call function in order to execute data
            //       signature recovery, how?
            let (multisig, wallets, asset) = setup().await;
        }
    }

    mod revert {

        use super::*;

        #[tokio::test]
        #[should_panic]
        async fn when_nonce_is_zero() {
            let (multisig, wallets, _) = setup().await;

            let to = Identity::Address(wallets.users[0].address().into());
            let value: u64 = 100;
            let data: Vec<u64> = vec![52, 45, 17];
            let b512 = B512 {
                bytes: vec![[0u8; 32], [0u8; 32]],
            };
            let signatures: Vec<B512> = vec![b512.clone(), b512.clone(), b512.clone()];

            execute_transaction(&multisig.contract, data, signatures, to, value).await;
        }

        #[tokio::test]
        #[should_panic]
        async fn when_unrecoverable_public_key() {
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            let to = Identity::Address(wallets.users[0].address().into());
            let value: u64 = 100;
            let data: Vec<u64> = vec![52, 45, 17];
            let b512 = B512 {
                bytes: vec![[0u8; 32], [0u8; 32]],
            };
            let signatures: Vec<B512> = vec![b512.clone(), b512.clone(), b512.clone()];

            constructor(&multisig.contract, 2, users.clone()).await;
            execute_transaction(&multisig.contract, data, signatures, to, value).await;
        }

        #[tokio::test]
        #[ignore]
        #[should_panic]
        async fn when_incorrect_signer_ordering() {
            // TODO: users must sign in order to pass ec_recover_address, how do we sign?
            //       wallets are randomly generated therefore we must hardcode instead or test may falsely pass
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            let to = Identity::Address(wallets.users[0].address().into());
            let value: u64 = 100;
            let data: Vec<u64> = vec![52, 45, 17];
            let b512_zero = B512 {
                bytes: vec![
                    *users.get(1).unwrap().identity,
                    *users.get(1).unwrap().identity,
                ],
            };
            let b512_one = B512 {
                bytes: vec![
                    *users.get(2).unwrap().identity,
                    *users.get(2).unwrap().identity,
                ],
            };
            let b512_two = B512 {
                bytes: vec![
                    *users.get(0).unwrap().identity,
                    *users.get(0).unwrap().identity,
                ],
            };
            let signatures: Vec<B512> = vec![b512_one, b512_zero, b512_two];

            constructor(&multisig.contract, 2, users.clone()).await;
            execute_transaction(&multisig.contract, data, signatures, to, value).await;
        }

        #[tokio::test]
        #[ignore]
        #[should_panic]
        async fn when_insufficient_approval_count() {
            // TODO: users must sign in order to pass ec_recover_address, how do we sign?
            //       wallets are randomly generated therefore we must hardcode instead or test may falsely pass
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            let to = Identity::Address(wallets.users[0].address().into());
            let value: u64 = 100;
            let data: Vec<u64> = vec![52, 45, 17];
            let b512_zero = B512 {
                bytes: vec![
                    *users.get(1).unwrap().identity,
                    *users.get(1).unwrap().identity,
                ],
            };
            let b512_one = B512 {
                bytes: vec![
                    *users.get(2).unwrap().identity,
                    *users.get(2).unwrap().identity,
                ],
            };
            let b512_two = B512 {
                bytes: vec![
                    *users.get(0).unwrap().identity,
                    *users.get(0).unwrap().identity,
                ],
            };
            let signatures: Vec<B512> = vec![b512_one, b512_zero, b512_two];

            constructor(&multisig.contract, 5, users.clone()).await;
            execute_transaction(&multisig.contract, data, signatures, to, value).await;
        }
    }
}

mod nonce {

    use super::*;

    mod success {

        use super::*;

        #[tokio::test]
        async fn returns_zero() {
            let (multisig, _, _) = setup().await;
            assert_eq!(0, nonce(&multisig.contract).await);
        }

        #[tokio::test]
        async fn returns_one() {
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            assert_eq!(0, nonce(&multisig.contract).await);

            constructor(&multisig.contract, 2, users.clone()).await;

            assert_eq!(1, nonce(&multisig.contract).await);
        }
    }
}

mod owner {

    use super::*;

    mod success {

        use super::*;

        #[tokio::test]
        async fn returns_info_for_existing_owner() {
            let (multisig, wallets, _) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            assert_eq!(
                Owner { weight: 0 },
                owner(&multisig.contract, &wallets.users[0]).await
            );
            assert_eq!(
                Owner { weight: 0 },
                owner(&multisig.contract, &wallets.users[1]).await
            );
            assert_eq!(
                Owner { weight: 0 },
                owner(&multisig.contract, &wallets.users[2]).await
            );

            constructor(&multisig.contract, 2, users.clone()).await;

            assert_eq!(
                Owner {
                    weight: users.get(0).unwrap().weight
                },
                owner(&multisig.contract, &wallets.users[0]).await
            );
            assert_eq!(
                Owner {
                    weight: users.get(1).unwrap().weight
                },
                owner(&multisig.contract, &wallets.users[1]).await
            );
            assert_eq!(
                Owner {
                    weight: users.get(2).unwrap().weight
                },
                owner(&multisig.contract, &wallets.users[2]).await
            );
        }

        #[tokio::test]
        async fn returns_info_for_non_owner() {
            let (multisig, wallets, _) = setup().await;
            assert_eq!(
                Owner { weight: 0 },
                owner(&multisig.contract, &wallets.users[0]).await
            );
        }
    }
}

mod transfer {

    use super::*;

    mod success {

        use super::*;

        #[tokio::test]
        #[ignore]
        async fn transfers() {
            // TODO: signature recovery, how?
            let (multisig, wallets, asset) = setup().await;
        }
    }

    mod revert {

        use super::*;

        #[tokio::test]
        #[should_panic]
        async fn when_nonce_is_zero() {
            let (multisig, wallets, asset) = setup().await;

            let to = Identity::Address(wallets.users[0].address().into());
            let value: u64 = 100;
            let data: Vec<u64> = vec![52, 45, 17];
            let b512 = B512 {
                bytes: vec![[0u8; 32], [0u8; 32]],
            };
            let signatures: Vec<B512> = vec![b512.clone(), b512.clone(), b512.clone()];

            transfer(asset.id, &multisig.contract, data, signatures, to, value).await;
        }

        #[tokio::test]
        #[should_panic]
        async fn when_transferring_more_than_contract_balance() {
            let (multisig, wallets, asset) = setup().await;

            let to = Identity::Address(wallets.users[0].address().into());
            let value: u64 = 100;
            let data: Vec<u64> = vec![52, 45, 17];
            let b512 = B512 {
                bytes: vec![[0u8; 32], [0u8; 32]],
            };
            let signatures: Vec<B512> = vec![b512.clone(), b512.clone(), b512.clone()];

            transfer(asset.id, &multisig.contract, data, signatures, to, value).await;
        }

        #[tokio::test]
        #[should_panic]
        async fn when_unrecoverable_public_key() {
            let (multisig, wallets, asset) = setup().await;
            let users = vec![
                User {
                    identity: wallets.users[0].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[1].address().into(),
                    weight: 1,
                },
                User {
                    identity: wallets.users[2].address().into(),
                    weight: 2,
                },
            ];

            let to = Identity::Address(wallets.users[0].address().into());
            let value: u64 = 0;
            let data: Vec<u64> = vec![52, 45, 17];
            let b512 = B512 {
                bytes: vec![[0u8; 32], [0u8; 32]],
            };
            let signatures: Vec<B512> = vec![b512.clone(), b512.clone(), b512.clone()];

            constructor(&multisig.contract, 2, users.clone()).await;
            transfer(asset.id, &multisig.contract, data, signatures, to, value).await;
        }

        #[tokio::test]
        #[ignore]
        #[should_panic]
        async fn when_incorrect_signer_ordering() {
            // TODO: users must sign in order to pass ec_recover_address, how do we sign?
            //       wallets are randomly generated therefore we must hardcode instead or test may falsely pass
            let (multisig, wallets, asset) = setup().await;
        }

        #[tokio::test]
        #[ignore]
        #[should_panic]
        async fn when_insufficient_approval_count() {
            // TODO: users must sign in order to pass ec_recover_address, how do we sign?
            //       wallets are randomly generated therefore we must hardcode instead or test may falsely pass
            let (multisig, wallets, asset) = setup().await;
        }
    }
}

mod transaction_hash {

    use super::*;

    mod success {
        use super::*;
        use fuels::core::abi_encoder::ABIEncoder;
        use fuels::core::Tokenizable;

        #[tokio::test]
        async fn returns_hash() {
            let (multisig, wallets, _) = setup().await;

            // Raw values
            let to = Identity::Address(wallets.users[0].address().into());
            let value: u64 = 100;
            let data: Vec<u64> = vec![52, 45, 17];
            let nonce: u64 = 42;
            let contract_id: [u8; 32] = multisig.id.into();

            // Lazy conversion, must be in the same order as the params in the definition of the Tx struct
            let mut result: Vec<u8> = Vec::new();
            result.extend(contract_id);
            result.extend(
                [
                    data[0].to_be_bytes(),
                    data[1].to_be_bytes(),
                    data[2].to_be_bytes(),
                ]
                .concat(),
            );
            result.extend(ABIEncoder::encode(&[to.clone().into_token()]).unwrap());
            result.extend(nonce.to_be_bytes());
            result.extend(value.to_be_bytes());

            let expected: [u8; 32] = Sha256::digest(result).into();
            let hashed_transaction =
                transaction_hash(&multisig.contract, data, nonce, to, value).await;

            assert_eq!(expected, hashed_transaction);
        }
    }
}
