use crate::utils::{
    abi_calls::{campaign_info, cancel_campaign, claim_pledges, create_campaign, pledge},
    test_helpers::{mint, setup},
    Identity,
};
use fuels::{signers::Signer, tx::AssetId};

mod success {

    use super::*;

    #[tokio::test]
    async fn claims() {
        let (author, user, asset, _, defaults) = setup().await;
        let beneficiary = Identity::Address(author.wallet.address());
        let deadline = 6;

        mint(
            &asset.contract,
            defaults.target_amount,
            user.wallet.address(),
        )
        .await;
        create_campaign(
            &author.contract,
            &defaults.asset_id,
            &beneficiary,
            deadline,
            defaults.target_amount,
        )
        .await;

        pledge(&user.contract, 1, &asset, defaults.target_amount).await;

        assert_eq!(
            0,
            author
                .wallet
                .get_asset_balance(&AssetId::from(*asset.id))
                .await
                .unwrap()
        );

        claim_pledges(&author.contract, 1).await;

        assert_eq!(
            defaults.target_amount,
            author
                .wallet
                .get_asset_balance(&AssetId::from(*asset.id))
                .await
                .unwrap()
        );
        assert_eq!(campaign_info(&author.contract, 1).await.value.claimed, true);
    }
}

mod revert {

    use super::*;

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn when_id_is_zero() {
        let (author, _, _, _, defaults) = setup().await;

        create_campaign(
            &author.contract,
            &defaults.asset_id,
            &defaults.beneficiary,
            defaults.deadline,
            defaults.target_amount,
        )
        .await;

        // Reverts
        claim_pledges(&author.contract, 0).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn when_id_is_greater_than_number_of_campaigns() {
        let (author, _, _, _, defaults) = setup().await;

        create_campaign(
            &author.contract,
            &defaults.asset_id,
            &defaults.beneficiary,
            defaults.deadline,
            defaults.target_amount,
        )
        .await;

        // Reverts
        claim_pledges(&author.contract, 100).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn when_sender_is_not_author() {
        let (author, user, _, _, defaults) = setup().await;

        create_campaign(
            &author.contract,
            &defaults.asset_id,
            &defaults.beneficiary,
            defaults.deadline,
            defaults.target_amount,
        )
        .await;

        // Reverts
        claim_pledges(&user.contract, 1).await;
    }

    // #[tokio::test]
    // #[should_panic(expected = "Revert(42)")]
    // async fn when_claiming_before_deadline() {
    //     let (author, user, asset, _, defaults) = setup().await;
    //     let deadline = 5;

    //     mint(&asset.contract, defaults.target_amount, user.wallet.address()).await;
    //     create_campaign(
    //         &author.contract,
    //         &defaults.asset_id,
    //         &defaults.beneficiary,
    //         deadline,
    //         defaults.target_amount,
    //     )
    //     .await;
    //     pledge(&user.contract, 1, &asset, defaults.target_amount).await;

    //     // TODO: shift block height to be before deadline

    //     // Reverts
    //     claim_pledges(&author.contract, 1).await;
    // }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn when_target_amount_is_not_reached() {
        let (author, _, _, _, defaults) = setup().await;
        let deadline = 1;

        create_campaign(
            &author.contract,
            &defaults.asset_id,
            &defaults.beneficiary,
            deadline,
            defaults.target_amount,
        )
        .await;

        // Reverts
        claim_pledges(&author.contract, 1).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn when_claiming_more_than_once() {
        let (author, user, asset, _, defaults) = setup().await;
        let deadline = 5;

        mint(
            &asset.contract,
            defaults.target_amount,
            user.wallet.address(),
        )
        .await;
        create_campaign(
            &author.contract,
            &defaults.asset_id,
            &defaults.beneficiary,
            deadline,
            defaults.target_amount,
        )
        .await;
        pledge(&user.contract, 1, &asset, defaults.target_amount).await;
        claim_pledges(&author.contract, 1).await;

        // Reverts
        claim_pledges(&author.contract, 1).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn when_cancelled() {
        let (author, user, asset, _, defaults) = setup().await;
        let deadline = 6;

        mint(
            &asset.contract,
            defaults.target_amount,
            user.wallet.address(),
        )
        .await;
        create_campaign(
            &author.contract,
            &defaults.asset_id,
            &defaults.beneficiary,
            deadline,
            defaults.target_amount,
        )
        .await;
        pledge(&user.contract, 1, &asset, defaults.target_amount).await;
        cancel_campaign(&author.contract, 1).await;

        // Reverts
        claim_pledges(&author.contract, 1).await;
    }
}
