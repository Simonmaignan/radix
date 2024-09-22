use scrypto::prelude::*;

#[blueprint]
mod token_sale {
    struct TokenSale {
        // Define what resources and data will be managed by TokenSale components
        useful_tokens_vault: FungibleVault,
        collected_xrd: FungibleVault,
        price_per_token: Decimal,
    }

    impl TokenSale {
        // Implement the functions and methods which will manage those resources and data
        pub fn instantiate_token_sale(price_per_token: Decimal) -> Global<TokenSale> {
            // Create a useful token
            let bucket_of_useful_token: FungibleBucket =
                ResourceBuilder::new_fungible(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Useful Token", locked;
                            "symbol" => "USEFUL", locked;
                        }
                    ))
                    .mint_initial_supply(100);
            Self {
                useful_tokens_vault: FungibleVault::with_bucket(bucket_of_useful_token),
                collected_xrd: FungibleVault::new(XRD),
                price_per_token: price_per_token,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn buy_useful_token(
            &mut self,
            mut payment: FungibleBucket,
        ) -> (FungibleBucket, FungibleBucket) {
            self.collected_xrd.put(payment.take(self.price_per_token));
            (self.useful_tokens_vault.take(1), payment)
        }
    }
}
