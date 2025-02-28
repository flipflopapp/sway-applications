use crate::utils::{
    abi_calls::{accept_arbiter, create_escrow, deposit, propose_arbiter, transfer_to_seller},
    test_helpers::{asset_amount, create_arbiter, create_asset, mint, setup},
};
use fuels::signers::Signer;

mod success {

    use super::*;

    #[tokio::test]
    async fn accepts_proposal() {
        let (arbiter, buyer, seller, defaults) = setup().await;
        let arbiter_obj = create_arbiter(
            arbiter.wallet.address(),
            defaults.asset_id,
            defaults.asset_amount,
        )
        .await;
        let asset = create_asset(defaults.asset_amount, defaults.asset_id).await;

        mint(
            &defaults.asset,
            seller.wallet.address(),
            defaults.asset_amount * 2,
        )
        .await;
        create_escrow(
            defaults.asset_amount,
            &arbiter_obj,
            &defaults.asset_id,
            vec![asset.clone(), asset.clone()],
            buyer.wallet.address(),
            &seller.contract,
            defaults.deadline,
        )
        .await;

        propose_arbiter(arbiter_obj, &seller.contract, 0).await;
        assert_eq!(0, asset_amount(&defaults.asset_id, &seller.wallet).await);

        accept_arbiter(&buyer.contract, 0).await;
        assert_eq!(
            defaults.asset_amount,
            asset_amount(&defaults.asset_id, &seller.wallet).await
        );
    }

    #[tokio::test]
    async fn accepts_proposal_in_two_escrows() {
        let (arbiter, buyer, seller, defaults) = setup().await;
        let arbiter_obj = create_arbiter(
            arbiter.wallet.address(),
            defaults.asset_id,
            defaults.asset_amount,
        )
        .await;
        let asset = create_asset(defaults.asset_amount, defaults.asset_id).await;

        mint(
            &defaults.asset,
            seller.wallet.address(),
            defaults.asset_amount * 4,
        )
        .await;

        create_escrow(
            defaults.asset_amount,
            &arbiter_obj,
            &defaults.asset_id,
            vec![asset.clone(), asset.clone()],
            buyer.wallet.address(),
            &seller.contract,
            defaults.deadline,
        )
        .await;
        create_escrow(
            defaults.asset_amount,
            &arbiter_obj,
            &defaults.asset_id,
            vec![asset.clone(), asset.clone()],
            buyer.wallet.address(),
            &seller.contract,
            defaults.deadline,
        )
        .await;

        assert_eq!(
            defaults.asset_amount * 2,
            asset_amount(&defaults.asset_id, &seller.wallet).await
        );

        propose_arbiter(arbiter_obj.clone(), &seller.contract, 0).await;
        assert_eq!(
            defaults.asset_amount,
            asset_amount(&defaults.asset_id, &seller.wallet).await
        );

        propose_arbiter(arbiter_obj, &seller.contract, 1).await;
        assert_eq!(0, asset_amount(&defaults.asset_id, &seller.wallet).await);

        accept_arbiter(&buyer.contract, 0).await;
        assert_eq!(
            defaults.asset_amount,
            asset_amount(&defaults.asset_id, &seller.wallet).await
        );

        accept_arbiter(&buyer.contract, 1).await;
        assert_eq!(
            defaults.asset_amount * 2,
            asset_amount(&defaults.asset_id, &seller.wallet).await
        );
    }
}

mod revert {

    use super::*;

    #[tokio::test]
    #[should_panic]
    async fn when_escrow_is_not_pending() {
        let (arbiter, buyer, seller, defaults) = setup().await;
        let arbiter_obj = create_arbiter(
            arbiter.wallet.address(),
            defaults.asset_id,
            defaults.asset_amount,
        )
        .await;
        let asset = create_asset(defaults.asset_amount, defaults.asset_id).await;

        mint(
            &defaults.asset,
            seller.wallet.address(),
            defaults.asset_amount * 2,
        )
        .await;
        mint(
            &defaults.asset,
            buyer.wallet.address(),
            defaults.asset_amount,
        )
        .await;

        create_escrow(
            defaults.asset_amount,
            &arbiter_obj,
            &defaults.asset_id,
            vec![asset.clone(), asset.clone()],
            buyer.wallet.address(),
            &seller.contract,
            defaults.deadline,
        )
        .await;
        deposit(
            defaults.asset_amount,
            &defaults.asset_id,
            &buyer.contract,
            0,
        )
        .await;
        transfer_to_seller(&buyer.contract, 0).await;
        accept_arbiter(&buyer.contract, 0).await;
    }

    #[tokio::test]
    #[should_panic]
    async fn when_caller_is_not_buyer() {
        let (arbiter, buyer, seller, defaults) = setup().await;
        let arbiter_obj = create_arbiter(
            arbiter.wallet.address(),
            defaults.asset_id,
            defaults.asset_amount,
        )
        .await;
        let asset = create_asset(defaults.asset_amount, defaults.asset_id).await;

        mint(
            &defaults.asset,
            seller.wallet.address(),
            defaults.asset_amount * 2,
        )
        .await;
        mint(
            &defaults.asset,
            buyer.wallet.address(),
            defaults.asset_amount,
        )
        .await;

        create_escrow(
            defaults.asset_amount,
            &arbiter_obj,
            &defaults.asset_id,
            vec![asset.clone(), asset.clone()],
            buyer.wallet.address(),
            &seller.contract,
            defaults.deadline,
        )
        .await;
        deposit(
            defaults.asset_amount,
            &defaults.asset_id,
            &buyer.contract,
            0,
        )
        .await;
        accept_arbiter(&seller.contract, 0).await;
    }

    #[tokio::test]
    #[should_panic]
    async fn when_arbiter_proposal_is_not_set() {
        let (arbiter, buyer, seller, defaults) = setup().await;
        let arbiter_obj = create_arbiter(
            arbiter.wallet.address(),
            defaults.asset_id,
            defaults.asset_amount,
        )
        .await;
        let asset = create_asset(defaults.asset_amount, defaults.asset_id).await;

        mint(
            &defaults.asset,
            seller.wallet.address(),
            defaults.asset_amount * 2,
        )
        .await;
        mint(
            &defaults.asset,
            buyer.wallet.address(),
            defaults.asset_amount,
        )
        .await;

        create_escrow(
            defaults.asset_amount,
            &arbiter_obj,
            &defaults.asset_id,
            vec![asset.clone(), asset.clone()],
            buyer.wallet.address(),
            &seller.contract,
            defaults.deadline,
        )
        .await;
        deposit(
            defaults.asset_amount,
            &defaults.asset_id,
            &buyer.contract,
            0,
        )
        .await;
        accept_arbiter(&buyer.contract, 0).await;
    }
}
