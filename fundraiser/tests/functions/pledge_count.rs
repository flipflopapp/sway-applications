use crate::utils::{
    abi_calls::{create_campaign, pledge, pledge_count},
    test_helpers::{mint, setup},
    Identity,
};
use fuels::signers::Signer;

mod success {

    use super::*;

    #[tokio::test]
    async fn returns_zero() {
        let (_, user, _, _, _) = setup().await;

        assert_eq!(
            0,
            pledge_count(&user.contract, Identity::Address(user.wallet.address())).await
        );
    }

    #[tokio::test]
    async fn returns_one() {
        let (author, user, asset, _, defaults) = setup().await;

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
            defaults.deadline,
            defaults.target_amount,
        )
        .await;

        pledge(&user.contract, 1, &asset, defaults.target_amount).await;
        assert_eq!(
            1,
            pledge_count(&user.contract, Identity::Address(user.wallet.address())).await
        );
    }
}
