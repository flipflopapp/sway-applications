use crate::utils::{
    abi_calls::{constructor, is_approved_for_all, set_approval_for_all},
    test_helpers::setup,
    Identity,
};
use fuels::signers::Signer;

mod success {

    use super::*;

    #[tokio::test]
    async fn sets_approval_for_all() {
        let (deploy_wallet, owner1, owner2) = setup().await;

        // constructor(false, &deploy_wallet.contract, &Option::None(), 1).await;
        let admin = Identity::Address(owner1.wallet.address());
        constructor(true, &deploy_wallet.contract, &admin, 1).await;

        let owner = Identity::Address(owner1.wallet.address());
        let operator = Identity::Address(owner2.wallet.address());

        assert_eq!(
            is_approved_for_all(&owner1.contract, &operator, &owner).await,
            false
        );

        set_approval_for_all(true, &owner1.contract, &operator).await;

        assert_eq!(
            is_approved_for_all(&owner1.contract, &operator, &owner).await,
            true
        );
    }

    #[tokio::test]
    async fn removes_approval_for_all() {
        let (deploy_wallet, owner1, owner2) = setup().await;

        // constructor(false, &deploy_wallet.contract, &Option::None(), 1).await;
        let admin = Identity::Address(owner1.wallet.address());
        constructor(true, &deploy_wallet.contract, &admin, 1).await;

        let owner = Identity::Address(owner1.wallet.address());
        let operator = Identity::Address(owner2.wallet.address());

        assert_eq!(
            is_approved_for_all(&owner1.contract, &operator, &owner).await,
            false
        );

        set_approval_for_all(true, &owner1.contract, &operator).await;

        assert_eq!(
            is_approved_for_all(&owner1.contract, &operator, &owner).await,
            true
        );

        set_approval_for_all(false, &owner1.contract, &operator).await;

        assert_eq!(
            is_approved_for_all(&owner1.contract, &operator, &owner).await,
            false
        );
    }
}
