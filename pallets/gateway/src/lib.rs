#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult,
                    pallet_prelude::*, traits::Currency};
use frame_support::sp_runtime::traits::Convert;
use frame_system::pallet_prelude::*;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use sp_runtime::traits::Zero;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;
pub use primitives::p_gateway::*;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


#[frame_support::pallet]
pub mod pallet {
    use primitives::Balance;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// currency to pay fees and hold balances
        type Currency: Currency<Self::AccountId>;

        /// amount converted to numbers
        type BalanceToNumber: Convert<BalanceOf<Self>, u128>;

        /// gateway node timed removal interval
        #[pallet::constant]
        type GatewayNodeTimedRemovalInterval: Get<Self::BlockNumber>;
        /// gateway node heartbeat reporting interval
        #[pallet::constant]
        type GatewayNodeHeartbeatInterval: Get<Self::BlockNumber>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    /// gateway node information
    #[pallet::storage]
    #[pallet::getter(fn gateway)]
    pub(super) type GatewayNodes<T: Config> = StorageMap<_,Twox64Concat,u64, GatewayNode<T::BlockNumber, T::AccountId>, OptionQuery>;

    /// gateway node index
    #[pallet::storage]
    #[pallet::getter(fn gateway_node_index)]
    pub(super) type GatewayNodeIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// number of resources
    #[pallet::storage]
    #[pallet::getter(fn gateway_node_count)]
    pub(super) type GatewayNodeCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// successfully registered resources
        /// [accountId, registration_time,index, peerId, ]
        RegisterGatewayNodeSuccess(T::AccountId, T::BlockNumber,u64, Vec<u8>),
        /// health check successfully [accountId, index, registration_time]
        HealthCheckSuccess(T::AccountId, u64, T::BlockNumber),
    }


    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// gateway node does not exist
        GatewayNodeNotFound,
        /// retry
        TryAgain,
        /// the owner of the gateway node does not belong to you
        GatewayNodeNotOwnedByYou,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: T::BlockNumber) -> Weight {
            // health examination
            if (now % T::GatewayNodeTimedRemovalInterval::get()).is_zero() {

                // get a list of gateway nodes
                let gateway_nodes = GatewayNodes::<T>::iter();

                for (i, mut gateway_node) in gateway_nodes {
                    // get the interval from the last heartbeat report
                    let duration = now - gateway_node.registration_time;
                    // Check if heartbeat interval is exceeded
                    if duration > T::GatewayNodeHeartbeatInterval::get() {
                        //remove gateway node
                        GatewayNodes::<T>::remove(gateway_node.index);
                        // reduce count
                        let count = GatewayNodeCount::<T>::get();
                        GatewayNodeCount::<T>::set(count - 1);
                    }
                }
            }
            0
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// register gateway node
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_gateway_node(
            account_id: OriginFor<T>,
            peer_id: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(account_id)?;
            // get gateway node index
            let index = GatewayNodeIndex::<T>::get();

            // get the current block height
            let block_number = <frame_system::Pallet<T>>::block_number();

            // gateway node information
            let gateway_node = GatewayNode::new(
                index, who.clone(), peer_id.clone(),block_number,
            );

            // increase gateway nodes
            GatewayNodes::<T>::insert(index, gateway_node.clone());
            // increase the total
            let count = GatewayNodeCount::<T>::get();
            GatewayNodeCount::<T>::set(count + 1);
            // index auto increment
            GatewayNodeIndex::<T>::set(index + 1);

            Self::deposit_event(Event::RegisterGatewayNodeSuccess(who,block_number,index, peer_id));
            Ok(())
        }

        /// gateway node heartbeat
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn heartbeat(
            origin: OriginFor<T>,
            gateway_node_index: u64,
        )-> DispatchResult {
            let who = ensure_signed(origin)?;

            // get gateway node
            ensure!(GatewayNodes::<T>::contains_key(gateway_node_index),Error::<T>::GatewayNodeNotFound);
            let mut gateway_node = GatewayNodes::<T>::get(gateway_node_index).unwrap();
            // determine whether it is me
            ensure!(who.clone() == gateway_node.account_id,Error::<T>::GatewayNodeNotOwnedByYou);
            // get the current block height
            let block_number = <frame_system::Pallet<T>>::block_number();

            // update gateway node registration time
            gateway_node.registration_time = block_number;

            // save the gateway node
            GatewayNodes::<T>::insert(gateway_node_index.clone(), gateway_node.clone());

            Self::deposit_event(Event::HealthCheckSuccess(who.clone(), gateway_node_index.clone(), block_number));
            Ok(())
        }
    }
}

