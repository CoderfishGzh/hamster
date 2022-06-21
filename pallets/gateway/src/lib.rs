#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{dispatch::DispatchResult,
                    pallet_prelude::*, traits::Currency};
use frame_support::sp_runtime::traits::Convert;
use frame_system::pallet_prelude::*;
use primitives::EraIndex;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use sp_runtime::traits::Zero;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;
pub use primitives::p_gateway::*;
pub use pallet_market::MarketInterface;
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

        type MarketInterface: MarketInterface<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    /// gateway node information
    #[pallet::storage]
    #[pallet::getter(fn gateway)]
    pub(super) type GatewayNodes<T: Config> = StorageMap<_,Twox64Concat,Vec<u8>, GatewayNode<T::BlockNumber, T::AccountId>, OptionQuery>;

    ///list of gateway nodes
    #[pallet::storage]
    #[pallet::getter(fn gateways)]
    pub(super) type Gateways<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn era)]
    pub(super) type Era<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// number of gateway nodes
    #[pallet::storage]
    #[pallet::getter(fn gateway_node_count)]
    pub(super) type GatewayNodeCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// The current era
    #[pallet::storage]
    #[pallet::getter(fn currenct_era)]
    pub(super) type CurrentEra<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Current era total online time
    #[pallet::storage]
    #[pallet::getter(fn current_era_online_time)]
    pub(super) type CurrentEraOnlineTime<T: Config> = StorageValue<_, u128, ValueQuery>;


    /// gateway node online time 
    #[pallet::storage]
    #[pallet::getter(fn gateway_node_online_time)]
    pub(super) type GatewayNodeOnlineTime<T: Config> = StorageMap<
        _,
        Twox64Concat,
        GatewayNode<T::BlockNumber, T::AccountId>,
        u128,
        OptionQuery,
    >;

    /// Gateway node points
    #[pallet::storage]
    #[pallet::getter(fn gateway_node_points)]
    pub(super) type GatewayNodePoints<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        u128,
        OptionQuery,
    >;

    /// Era total points
    #[pallet::storage]
    #[pallet::getter(fn era_total_points)]
    pub(super) type EraTotalPoints<T: Config> = StorageMap<
        _,
        Twox64Concat,
        EraIndex,
        u128,
        OptionQuery,
    >;

    /// Gateway current total points
    #[pallet::storage]
    #[pallet::getter(fn currenct_total_points)]
    pub(super) type CurrentTotalPoints<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// Online time per era, per gateway node
    #[pallet::storage]
    #[pallet::getter(fn online_time_era_gateway)]
    pub(super) type OnlineTimeEraGateway<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat, EraIndex,
        Twox64Concat, T::AccountId,
        u128,
        OptionQuery,
    >;

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub gateway: Vec<(Vec<u8>,GatewayNode<T::BlockNumber, T::AccountId>)>,
        pub gateway_node_count: u64,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                gateway: Default::default(),
                gateway_node_count: Default::default(),
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <GatewayNodeCount<T>>::put(&self.gateway_node_count);
            for (a, b) in &self.gateway {
                <GatewayNodes<T>>::insert(a, b);
            }
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// successfully registered resources
        /// [accountId, registration_time, peerId, ]
        RegisterGatewayNodeSuccess(T::AccountId, T::BlockNumber, Vec<u8>),
        /// health check successfully [accountId, registration_time]
        HealthCheckSuccess(T::AccountId, T::BlockNumber),

        ClearPoinstSuccess,
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

        GatewayNodeNotStakingAccoutId,

        NotEnoughAmount,

        BingStakingInfoFailed,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: T::BlockNumber) -> Weight {
            // get a list of online gateway nodes
            let gateway_nodes = GatewayNodes::<T>::iter();
            // Accumulated points
            let mut total_points: u128 = 0;
            // Update the gateway node pointc
            for(_, ref gateway_node) in gateway_nodes {
                total_points += 10;
                // Determine the gateway node has been recorded
                if GatewayNodePoints::<T>::contains_key(gateway_node.account_id.clone()) {
                    // Get the info
                    let mut gateway_node_points = GatewayNodePoints::<T>::get(gateway_node.account_id.clone()).unwrap();
                    // Update the info
                    gateway_node_points += 10;
                    GatewayNodePoints::<T>::insert(gateway_node.account_id.clone(), gateway_node_points);
                } else {
                    GatewayNodePoints::<T>::insert(gateway_node.account_id.clone(), 10);
                }
            }
            // Update the total points
            let mut _total_points = CurrentTotalPoints::<T>::get();
            _total_points += total_points;
            CurrentTotalPoints::<T>::set(_total_points);

            // health examination
            if (now % T::GatewayNodeTimedRemovalInterval::get()).is_zero() {

                // get a list of gateway nodes
                let gateway_nodes = GatewayNodes::<T>::iter();

                for (i, mut gateway_node) in gateway_nodes {
                    // get the interval from the last heartbeat report
                    let duration = now - gateway_node.registration_time;
                    // Check if heartbeat interval is exceeded
                    if duration > T::GatewayNodeHeartbeatInterval::get() {
                        let peer_id = gateway_node.peer_id;
                        let mut peerIds = Gateways::<T>::get();
                        if let Ok(index) = peerIds.binary_search(&peer_id){
                            peerIds.remove(index);
                            Gateways::<T>::put(peerIds);
                        }
                        //remove gateway node
                        GatewayNodes::<T>::remove(peer_id);
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

            // // The gateway must have a staking account in order to register
            // ensure!(
            //     T::MarketInterface::staking_accountid_exit(who.clone()),
            //     Error::<T>::GatewayNodeNotStakingAccoutId,
            // ); 

            // // get the staking info 
            // let mut staking_info = T::MarketInterface::staking_info(who.clone());
            // // gateway Pledge  
            // // Todo，the amount now is 100
            // ensure!(
            //     staking_info.lock_amount(100),
            //     Error::<T>::NotEnoughAmount,
            // );
            // // update staking info
            // T::MarketInterface::updata_staking_info(who.clone(), staking_info);

            // Binding the staking info
            if !Self::binding_staking_info(who.clone()) {
                return Err(Error::<T>::BingStakingInfoFailed.into());
            }

            // get the current block height
            let block_number = <frame_system::Pallet<T>>::block_number();

            // gateway node information
            let gateway_node = GatewayNode::new(
                who.clone(), peer_id.clone(),block_number,
            );

            // increase gateway nodes
            GatewayNodes::<T>::insert(peer_id.clone(), gateway_node.clone());

            let mut peerIds = Gateways::<T>::get();
            if let Err(index) = peerIds.binary_search(&peer_id){
                peerIds.insert(index,peer_id.clone());
                Gateways::<T>::put(peerIds);
            }
            // increase the total
            let count = GatewayNodeCount::<T>::get();
            GatewayNodeCount::<T>::set(count + 1);

            Self::deposit_event(Event::RegisterGatewayNodeSuccess(who,block_number, gateway_node.peer_id));
            Ok(())
        }
        
        /// gateway node heartbeat
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn heartbeat(
            origin: OriginFor<T>,
            peer_id: Vec<u8>,
        )-> DispatchResult {
            let who = ensure_signed(origin)?;
            // get gateway node
            ensure!(GatewayNodes::<T>::contains_key(peer_id.clone()),Error::<T>::GatewayNodeNotFound);
            let mut gateway_node = GatewayNodes::<T>::get(peer_id.clone()).unwrap();
            // determine whether it is me
            ensure!(who.clone() == gateway_node.account_id,Error::<T>::GatewayNodeNotOwnedByYou);
            // get the current block height
            let block_number = <frame_system::Pallet<T>>::block_number();

            // update gateway node registration time
            gateway_node.registration_time = block_number;

            // save the gateway node
            GatewayNodes::<T>::insert(peer_id, gateway_node.clone());

            Self::deposit_event(Event::HealthCheckSuccess(who.clone(), block_number));
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    // Binding staking information
    pub fn binding_staking_info(who: T::AccountId) -> bool {
        // determine who has already binding
        if !T::MarketInterface::staking_accountid_exit(who.clone()) {
            return false;
        }
        // has binding
        // Get the staking info
        let mut staking_info = T::MarketInterface::staking_info(who.clone());
        // Gateway Pledge
        // todo: the amount now is 100
        if !staking_info.lock_amount(100) {
            return false;
        }
        // Update the staking info
        T::MarketInterface::updata_staking_info(who.clone(), staking_info);
        true
    }
}

impl <T: Config> GatewayInterface for Pallet<T> {

    // 计算gateway在线时间，并且记录在OnlineTimeEraGateway上面
    // fn calculate_online_time(index : EraIndex) {
    //     let gateway_nodes_online_times = GatewayNodeOnlineTime::<T>::iter();
    //     // Push GatewayNodeOnlineTime data to the OnlineTimeEraGateway
    //     for (gateway_node, online_time) in gateway_nodes_online_times {
    //         OnlineTimeEraGateway::<T>::insert(index, gateway_node.account_id.clone(), online_time);
    //         // Remove the data of GatewayNodeOnlineTime
    //         // GatewayNodeOnlineTime::<T>::remove(gateway_node.clone());
    //     }
    // }

    // // 计算gateway 当前时代每个结点的得分
    // // todo bug's here
    // fn compute_gateways_points() {
    //     let gateway_node_onlinetime = GatewayNodeOnlineTime::<T>::iter();
    //     for (gateway_node, onlintime) in gateway_node_onlinetime {
    //         T::MarketInterface::compute_gateways_points(
    //             gateway_node.clone().account_id,
    //             onlintime,
    //         );
    //     }
    // }

    // compute gateway reward
    // input:
    //  -total_reward: u128
    fn compute_gateways_reward(total_reward: u128, index: EraIndex) {
        let total_points = CurrentTotalPoints::<T>::get();
        // Get the gateway node and its points
        let gateway_points = GatewayNodePoints::<T>::iter();
        for (who, point) in gateway_points {
            // compute the ratio about gateway reward
            let ratio = point / total_points;
            let reward = ratio * total_reward;
            T::MarketInterface::save_gateway_reward(who.clone(), reward, index);
        }
    }
    // Todo
    // has bug
    fn clear_points_info(index: EraIndex) {
        let current_total_points = CurrentTotalPoints::<T>::get();
        EraTotalPoints::<T>::insert(index, current_total_points);
        CurrentTotalPoints::<T>::set(0);
        // todo
        // use false struct
        let gateway_node_points = GatewayNodePoints::<T>::iter();
        for (who, _) in gateway_node_points {
           GatewayNodePoints::<T>::remove(who.clone());
        }

        Self::deposit_event(Event::ClearPoinstSuccess);
    }
}