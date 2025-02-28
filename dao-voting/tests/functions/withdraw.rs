use crate::utils::{
    abi_calls::{balance, constructor, deposit, user_balance, withdraw},
    test_helpers::{mint, setup},
    Identity,
};
use fuels::{prelude::CallParameters, signers::Signer, tx::AssetId};

mod success {
    use super::*;

    #[tokio::test]
    async fn user_can_withdraw() {
        let (_gov_token, gov_token_id, deployer, user, asset_amount) = setup().await;

        mint(
            &deployer.gov_token.as_ref().unwrap(),
            asset_amount,
            user.wallet.address(),
        )
        .await;

        constructor(&deployer.dao_voting, gov_token_id).await;

        let call_params = CallParameters::new(
            Some(asset_amount),
            Some(AssetId::from(*gov_token_id)),
            Some(100_000),
        );
        deposit(&user.dao_voting, call_params).await;

        assert_eq!(balance(&user.dao_voting).await, asset_amount);

        assert_eq!(
            user_balance(&user.dao_voting, Identity::Address(user.wallet.address())).await,
            asset_amount
        );

        withdraw(&user.dao_voting, asset_amount).await;

        assert_eq!(
            user_balance(&user.dao_voting, Identity::Address(user.wallet.address())).await,
            0
        );

        assert_eq!(balance(&user.dao_voting).await, 0);
    }
}

mod revert {
    use super::*;

    #[tokio::test]
    #[should_panic]
    async fn panics_on_withdraw_zero() {
        let (_gov_token, gov_token_id, deployer, user, asset_amount) = setup().await;

        mint(
            &deployer.gov_token.as_ref().unwrap(),
            asset_amount,
            user.wallet.address(),
        )
        .await;

        constructor(&deployer.dao_voting, gov_token_id).await;

        let call_params = CallParameters::new(
            Some(asset_amount),
            Some(AssetId::from(*gov_token_id)),
            Some(100_000),
        );
        deposit(&user.dao_voting, call_params).await;
        withdraw(&user.dao_voting, 0).await;
    }

    #[tokio::test]
    #[should_panic]
    async fn panics_on_not_enough_assets() {
        let (_gov_token, gov_token_id, deployer, user, asset_amount) = setup().await;

        mint(
            &deployer.gov_token.as_ref().unwrap(),
            asset_amount,
            user.wallet.address(),
        )
        .await;

        constructor(&deployer.dao_voting, gov_token_id).await;

        let call_params = CallParameters::new(
            Some(asset_amount),
            Some(AssetId::from(*gov_token_id)),
            Some(100_000),
        );
        deposit(&user.dao_voting, call_params).await;
        withdraw(&user.dao_voting, asset_amount * 100).await;
    }
}
