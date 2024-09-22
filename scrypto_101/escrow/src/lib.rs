use scrypto::prelude::*;

#[blueprint]
mod escrow {
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
            todo!();
        }

        pub fn exchange(&mut self, bucket_of_resource: Bucket) -> Bucket {
            todo!();
        }

        pub fn withdraw_resource(&mut self, escrow_nft: NonFungibleBucket) -> Bucket {
            todo!();
        }

        pub fn cancel_escrow(&mut self, escrow_nft: NonFungibleBucket) -> Bucket {
            todo!();
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
