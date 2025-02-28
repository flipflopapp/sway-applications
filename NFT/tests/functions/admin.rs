use crate::utils::{
    abi_calls::{admin, constructor, set_admin},
    test_helpers::setup,
    Identity,
};
use fuels::signers::Signer;

mod success {

    use super::*;

    #[tokio::test]
    async fn gets_admin() {
        let (deploy_wallet, owner1, _owner2) = setup().await;

        // let new_admin = Option::Some(Identity::Address(owner1.wallet.address()));
        let new_admin = Identity::Address(owner1.wallet.address());
        constructor(true, &deploy_wallet.contract, &new_admin, 1).await;

        assert_eq!(admin(&owner1.contract).await, new_admin);
    }

    #[tokio::test]
    async fn gets_admin_after_change() {
        let (deploy_wallet, owner1, owner2) = setup().await;

        // let new_admin = Option::Some(Identity::Address(owner1.wallet.address()));
        let new_admin = Identity::Address(owner1.wallet.address());
        constructor(true, &deploy_wallet.contract, &new_admin, 1).await;

        assert_eq!(admin(&owner1.contract).await, new_admin);

        // let new_admin = Option::Some(minter.clone());
        let new_admin = Identity::Address(owner2.wallet.address());
        set_admin(&owner1.contract, &new_admin).await;

        assert_eq!(admin(&owner1.contract).await, new_admin);
    }
}

mod reverts {

    use super::*;

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_admin_not_set() {
        let (_deploy_wallet, owner1, _owner2) = setup().await;

        admin(&owner1.contract).await;
    }
}
