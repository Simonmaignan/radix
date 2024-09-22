use scrypto::prelude::*;

#[blueprint]
mod radiswap_module {
    struct Radiswap {
        vault_a: FungibleVault,
        vault_b: FungibleVault,
        pool_units_resource_manager: ResourceManager,
        fee: Decimal,
    }

    impl Radiswap {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate_radiswap(
            bucket_a: FungibleBucket,
            bucket_b: FungibleBucket,
            fee: Decimal,
        ) -> (Global<Radiswap>, FungibleBucket) {
            // Assert non empty bucket A and B
            assert!(
                !bucket_a.is_empty() && !bucket_b.is_empty(),
                "You must pass in an initial supply of each token."
            );

            // Assert fee is between 0 and 1
            assert!(
                fee >= dec!("0") && fee <= dec!("1"),
                "Invalid fee; Must be between 0.0 and 1.0."
            );

            // Create component virtual badge
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Radiswap::blueprint_id());

            // Pool Unit Resource Manager
            let pool_unit: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Pool Units", locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                ))
                .mint_initial_supply(100);

            // Instantiate a Radiswap component
            let radiswap: Global<Radiswap> = Self {
                vault_a: FungibleVault::with_bucket(bucket_a),
                vault_b: FungibleVault::with_bucket(bucket_b),
                pool_units_resource_manager: pool_unit.resource_manager(),
                fee: fee,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .globalize();

            (radiswap, pool_unit)
        }

        pub fn swap(&mut self, input_tokens: FungibleBucket) -> FungibleBucket {
            let (input_token_vault, output_token_vault): (&mut FungibleVault, &mut FungibleVault) =
                if input_tokens.resource_address() == self.vault_a.resource_address() {
                    (&mut self.vault_a, &mut self.vault_b)
                } else if input_tokens.resource_address() == self.vault_b.resource_address() {
                    (&mut self.vault_b, &mut self.vault_a)
                } else {
                    panic!("The given input token does not belong to this liquidity pool.")
                };
            let r: Decimal = dec!("1") - self.fee;
            let amount_input: Decimal = input_tokens.amount();
            let amount_output: Decimal = (output_token_vault.amount() * r * amount_input)
                / (input_token_vault.amount() + r * amount_input);

            input_token_vault.put(input_tokens);

            output_token_vault.take(amount_output)
        }

        pub fn add_liquidity(
            &mut self,
            bucket_a: FungibleBucket,
            bucket_b: FungibleBucket,
        ) -> (FungibleBucket, FungibleBucket, FungibleBucket) {
            let (mut bucket_a, mut bucket_b): (FungibleBucket, FungibleBucket) =
                if (bucket_a.resource_address() == self.vault_a.resource_address())
                    && (bucket_b.resource_address() == self.vault_b.resource_address())
                {
                    (bucket_a, bucket_b)
                } else if (bucket_a.resource_address() == self.vault_b.resource_address())
                    && (bucket_b.resource_address() == self.vault_a.resource_address())
                {
                    (bucket_b, bucket_a)
                } else {
                    panic!("One of the token does not belong to the pool!")
                };

            let dm: Decimal = bucket_a.amount();
            let dn: Decimal = bucket_b.amount();
            let m: Decimal = self.vault_a.amount();
            let n: Decimal = self.vault_b.amount();

            // Calculate the amount of token which will be taken from bucket_a and b
            let (amount_a, amount_b): (Decimal, Decimal) =
                if (m == Decimal::zero()) | (n == Decimal::zero()) | ((m / n) == (dm / dn)) {
                    (dm, dn)
                } else if (m / n) < (dm / dn) {
                    (dn * m / n, dn)
                } else {
                    (dm, dn * n / m)
                };

            // Depositing the amount of tokens calculated into the liquidity pool
            self.vault_a.put(bucket_a.take(amount_a));
            self.vault_b.put(bucket_b.take(amount_b));

            // Mint pool units tokens to the liquidity provider
            let pool_units_amount: Decimal =
                if self.pool_units_resource_manager.total_supply().unwrap() == Decimal::zero() {
                    dec!("100")
                } else {
                    amount_a * self.pool_units_resource_manager.total_supply().unwrap() / m
                };
            let pool_units: FungibleBucket = self
                .pool_units_resource_manager
                .mint(pool_units_amount)
                .as_fungible();

            (bucket_a, bucket_b, pool_units)
        }

        pub fn removing_liquidity(
            &mut self,
            pool_units: FungibleBucket,
        ) -> (FungibleBucket, FungibleBucket) {
            assert!(
                pool_units.resource_address() == self.pool_units_resource_manager.address(),
                "Wrong token passed in."
            );
            let share: Decimal =
                pool_units.amount() / self.pool_units_resource_manager.total_supply().unwrap();

            pool_units.burn();

            (
                self.vault_a.take(self.vault_a.amount() * share),
                self.vault_b.take(self.vault_b.amount() * share),
            )
        }
    }
}
