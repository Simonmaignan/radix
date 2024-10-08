use scrypto::prelude::*;

#[blueprint]
mod escrow {
    // enable_method_auth! {
    // methods {
    //     exchange => PUBLIC;
    //     withdraw_resource => restrict_to: [OWNER];
    //     cancel_escrow => restrict_to: [OWNER];
    // }
    // }
    struct Escrow {
        requested_resource: EscrowResourceSpecifier,
        offered_resource: Vault,
        requested_resource_vault: Vault,
        escrow_nft: ResourceAddress,
    }

    impl Escrow {
        pub fn instantiate_escrow(
            requested_resource: EscrowResourceSpecifier,
            offered_resource: Bucket,
        ) -> (Global<Escrow>, NonFungibleBucket) {
            let escrow_badge: NonFungibleBucket =
                ResourceBuilder::new_integer_non_fungible::<EscrowBadge>(OwnerRole::None)
                    .metadata(metadata!(init{"name" => "Escrow Badge", locked;}))
                    .mint_initial_supply(vec![(
                        IntegerNonFungibleLocalId::new(1),
                        EscrowBadge {
                            offered_resource: offered_resource.resource_address(),
                        },
                    )]);

            let owner_access_rule: AccessRule = rule!(require(escrow_badge.resource_address()));

            let escrow_inst: Global<Escrow> = Self {
                offered_resource: Vault::with_bucket(offered_resource),
                requested_resource_vault: Vault::new(requested_resource.get_resource_address()),
                requested_resource: requested_resource,
                escrow_nft: escrow_badge.resource_address(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(owner_access_rule))
            .metadata(metadata!(roles {
                metadata_setter => OWNER;
                metadata_setter_updater => OWNER;
                metadata_locker => OWNER;
                metadata_locker_updater => OWNER;
            },
            init {
                "name" => "Escrow Component", locked;
                "description" => "A component that allows trustless exchange.", locked;
            }))
            .globalize();

            (escrow_inst, escrow_badge)
        }

        pub fn exchange(&mut self, bucket_of_resource: Bucket) -> Bucket {
            assert_eq!(
                bucket_of_resource.resource_address(), self.requested_resource_vault.resource_address(),
                "The provided resource address for exchange does not match the requested resource address"
            );
            match &self.requested_resource {
                EscrowResourceSpecifier::Fungible {
                    resource_address: _,
                    amount,
                } => {
                    assert_eq!(bucket_of_resource.amount(), *amount, "The passed in fungible bucket amount does not match with the requested amount.");
                }
                EscrowResourceSpecifier::NonFungible {
                    resource_address: _,
                    non_fungible_local_id,
                } => assert_eq!(
                    *non_fungible_local_id,
                    bucket_of_resource.as_non_fungible().non_fungible_local_id(), "The passed in non fungible local id does not match with the requested non fungible local id."
                ),
            }

            self.requested_resource_vault.put(bucket_of_resource);

            self.offered_resource.take_all()
        }

        pub fn withdraw_resource(&mut self, escrow_nft: NonFungibleBucket) -> Bucket {
            assert_eq!(escrow_nft.resource_address(), self.escrow_nft, "The passed in escrow NFT badge does not match the required one to withdraw the resource.");
            assert!(
                !self.requested_resource_vault.is_empty(),
                "The requested resource vault is empty. The exchange has not taken place yet."
            );
            self.requested_resource_vault.take_all()
        }

        pub fn cancel_escrow(&mut self, escrow_nft: NonFungibleBucket) -> Bucket {
            assert_eq!(escrow_nft.resource_address(), self.escrow_nft, "The passed in escrow NFT badge does not match the required one to withdraw the resource.");
            assert!(
                !self.offered_resource.is_empty(),
                "The offered resource vault is empty. The exchange has already taken place. You can withdraw the requested resource."
            );
            self.offered_resource.take_all()
        }
    }
}

// Types //

#[derive(ScryptoSbor, Clone)]
pub enum EscrowResourceSpecifier {
    Fungible {
        resource_address: ResourceAddress,
        amount: Decimal,
    },
    NonFungible {
        resource_address: ResourceAddress,
        non_fungible_local_id: NonFungibleLocalId,
    },
}

impl EscrowResourceSpecifier {
    pub fn get_resource_address(&self) -> ResourceAddress {
        match self {
            Self::Fungible {
                resource_address, ..
            }
            | Self::NonFungible {
                resource_address, ..
            } => *resource_address,
        }
    }
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct EscrowBadge {
    offered_resource: ResourceAddress,
}
