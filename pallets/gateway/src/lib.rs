#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::sp_runtime::traits::Convert;

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
pub use primitives::p_chunkcycle::*;
pub use primitives::p_gateway::*;
pub use primitives::p_market::*;
use primitives::EraIndex;
use sp_runtime::traits::Zero;
use sp_runtime::{DispatchResultWithInfo, Perbill};
use sp_std::vec::Vec;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
const GATEWAY_PDGE_AMOUNT: u128 = 100;
const GATEWAY_LIMIT: u64 = 1000;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// currency to pay fees and hold balances
        type Currency: Currency<Self::AccountId>;

        /// digital transfer amount
        type NumberToBalance: Convert<u128, BalanceOf<Self>>;
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
    pub(super) type GatewayNodes<T: Config> = StorageMap<
        _,
        Twox64Concat,
        Vec<u8>,
        GatewayNode<T::BlockNumber, T::AccountId>,
        OptionQuery,
    >;

    ///list of gateway nodes
    #[pallet::storage]
    #[pallet::getter(fn gateways)]
    pub(super) type Gateways<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

    /// Account and peer id map
    /// Use to map the account to the peer id
    #[pallet::storage]
    #[pallet::getter(fn account_peerid_map)]
    pub(super) type AccountPeerMap<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<Vec<u8>>, OptionQuery>;

    /// number of gateway nodes
    #[pallet::storage]
    #[pallet::getter(fn gateway_node_count)]
    pub(super) type GatewayNodeCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Gateway node points
    #[pallet::storage]
    #[pallet::getter(fn gateway_node_points)]
    pub(super) type GatewayNodePoints<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, u128, OptionQuery>;

    /// Era total points
    #[pallet::storage]
    #[pallet::getter(fn era_total_points)]
    pub(super) type EraTotalPoints<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, u128, OptionQuery>;

    /// Gateway current total points
    #[pallet::storage]
    #[pallet::getter(fn currenct_total_points)]
    pub(super) type CurrentTotalPoints<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// This is set to v1.0.0 for new networks.
    #[pallet::storage]
    #[pallet::getter(fn storage_version)]
    pub(crate) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub gateway: Vec<(Vec<u8>, GatewayNode<T::BlockNumber, T::AccountId>)>,
        pub gateway_node_count: u64,
        pub account_peer_map: Vec<(T::AccountId, Vec<Vec<u8>>)>,
        pub gateways: Vec<Vec<u8>>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                gateway: Default::default(),
                gateway_node_count: Default::default(),
                account_peer_map: Default::default(),
                gateways: Default::default(),
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
            for (account, peerids) in &self.account_peer_map {
                <AccountPeerMap<T>>::insert(account, peerids);
            }

            <Gateways<T>>::set(self.gateways.clone());
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

        CurrentTotalPoints(u128),

        RatioAndReward(Perbill, u128),

        WithdrawGatewaySuccess(T::AccountId, Vec<u8>),

        ClearGatewayInfoSuccess(T::AccountId, Vec<u8>),
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

        GatewayNodeAlreadyExit,

        StakingNotExit,

        LockAmountFailed,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: T::BlockNumber) -> Weight {
            // health examination
            if (now % T::GatewayNodeTimedRemovalInterval::get()).is_zero() {
                // get a list of gateway nodes
                let gateway_nodes = GatewayNodes::<T>::iter();

                for (_, gateway_node) in gateway_nodes {
                    // get the interval from the last heartbeat report
                    let duration = now - gateway_node.registration_time;
                    // Check if heartbeat interval is exceeded
                    if duration > T::GatewayNodeHeartbeatInterval::get() {
                        Pallet::<T>::offline_gateway_node(
                            gateway_node.account_id.clone(),
                            gateway_node.peer_id.clone(),
                        );
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
        pub fn register_gateway_node(account_id: OriginFor<T>, peer_id: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(account_id)?;

            // 1. check the gateway node nus now < 1000
            let gateway_node_count = GatewayNodeCount::<T>::get();
            if gateway_node_count >= GATEWAY_LIMIT {
                return Err(Error::<T>::TryAgain.into());
            }

            // 2. check the gatway node is not registered
            ensure!(
                !GatewayNodes::<T>::contains_key(peer_id.clone()),
                Error::<T>::GatewayNodeAlreadyExit
            );

            // 3. check the user has staking
            ensure!(
                T::MarketInterface::staking_exit(who.clone()),
                Error::<T>::StakingNotExit
            );

            // 4. lock the staking amount
            ensure!(
                T::MarketInterface::change_stake_amount(
                    who.clone(),
                    ChangeAmountType::Lock,
                    100_000_000_000_000,
                    MarketUserStatus::Gateway
                ),
                Error::<T>::LockAmountFailed
            );

            // 5. deal the change of gateway node
            // get the current block height
            let block_number = <frame_system::Pallet<T>>::block_number();

            // gateway node information
            let gateway_node = GatewayNode::new(who.clone(), peer_id.clone(), block_number);

            // increase gateway nodes
            GatewayNodes::<T>::insert(peer_id.clone(), gateway_node.clone());

            let mut peer_ids = Gateways::<T>::get();

            // get the insert index of the peer id
            if let Err(index) = peer_ids.binary_search(&peer_id) {
                peer_ids.insert(index, peer_id.clone());
                // Update the peer id list
                Gateways::<T>::put(peer_ids);
            }

            // update the Account Peer Map
            Self::update_account_peer_map(who.clone(), peer_id.clone());

            // update the gateway node coutn
            GatewayNodeCount::<T>::set(gateway_node_count + 1);

            Self::deposit_event(Event::RegisterGatewayNodeSuccess(
                who,
                block_number,
                gateway_node.peer_id,
            ));

            Ok(())
        }

        /// gateway node heartbeat
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn heartbeat(origin: OriginFor<T>, peer_id: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // get gateway node
            ensure!(
                GatewayNodes::<T>::contains_key(peer_id.clone()),
                Error::<T>::GatewayNodeNotFound
            );
            let mut gateway_node = GatewayNodes::<T>::get(peer_id.clone()).unwrap();
            // determine whether it is me
            ensure!(
                who.clone() == gateway_node.account_id,
                Error::<T>::GatewayNodeNotOwnedByYou
            );
            // get the current block height
            let block_number = <frame_system::Pallet<T>>::block_number();

            // update gateway node registration time
            gateway_node.registration_time = block_number;

            // save the gateway node
            GatewayNodes::<T>::insert(peer_id, gateway_node.clone());

            Self::deposit_event(Event::HealthCheckSuccess(who.clone(), block_number));
            Ok(())
        }

        /// Take the specified peer offline
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn offline(account_id: OriginFor<T>, peer_id: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(account_id)?;

            // 1. check the gateway node exit
            ensure!(
                GatewayNodes::<T>::contains_key(peer_id.clone()),
                Error::<T>::GatewayNodeNotFound
            );

            // 2. check the gateway node is owned to you
            ensure!(
                GatewayNodes::<T>::get(peer_id.clone()).unwrap().account_id == who.clone(),
                Error::<T>::GatewayNodeNotOwnedByYou
            );

            // 3. deal the change of gateway node
            Self::offline_gateway_node(who.clone(), peer_id.clone());

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn offline_gateway_node(who: T::AccountId, peer_id: Vec<u8>) {
        // 1. update the gateway node count
        let gateway_node_count = GatewayNodeCount::<T>::get();
        GatewayNodeCount::<T>::set(gateway_node_count.saturating_sub(1));

        // 2. update the gateways list
        let mut peer_ids = Gateways::<T>::get();
        if let Ok(index) = peer_ids.binary_search(&peer_id) {
            peer_ids.remove(index);
            Gateways::<T>::put(peer_ids);
        }

        // 3. update the GatewayNode
        GatewayNodes::<T>::remove(peer_id.clone());

        // 4. update the Account Peer Map
        let mut account_peer_list = AccountPeerMap::<T>::get(who.clone()).unwrap();
        if let Ok(index) = account_peer_list.binary_search(&peer_id) {
            account_peer_list.remove(index);
        }
        if account_peer_list.is_empty() {
            AccountPeerMap::<T>::remove(who.clone());
        } else {
            AccountPeerMap::<T>::insert(who.clone(), account_peer_list);
        }

        // 5. unlock the staking amount
        T::MarketInterface::change_stake_amount(
            who.clone(),
            ChangeAmountType::Unlock,
            100_000_000_000_000,
            MarketUserStatus::Gateway,
        );
    }

    pub fn update_account_peer_map(who: T::AccountId, peer_id: Vec<u8>) {
        let mut account_peer_map: Vec<Vec<u8>>;

        if AccountPeerMap::<T>::contains_key(who.clone()) {
            account_peer_map = AccountPeerMap::<T>::get(who.clone()).unwrap();
        } else {
            account_peer_map = Vec::new();
        }

        if let Err(index) = account_peer_map.binary_search(&peer_id) {
            account_peer_map.insert(index, peer_id.clone());
        }

        AccountPeerMap::<T>::insert(who.clone(), account_peer_map);
    }

    // Binding staking information
    pub fn binding_staking_info(who: T::AccountId) -> bool {
        // determine who has already binding
        if !T::MarketInterface::staking_accountid_exit(who.clone()) {
            return false;
        }
        // Get the staking info
        let mut staking_info = T::MarketInterface::staking_info(who.clone());
        // Gateway Pledge
        // todo: the amount now is 100
        if !staking_info.lock_amount(GATEWAY_PDGE_AMOUNT) {
            return false;
        }
        // Update the staking info
        T::MarketInterface::updata_staking_info(who.clone(), staking_info);
        true
    }
}

impl<T: Config> GatewayInterface<<T as frame_system::Config>::AccountId> for Pallet<T> {
    /// TODO: let the compute into the pallet market
    /// compute_gateway_reward
    /// Calculate the reward for each node, though func:  save_gateway_reward that saves the reward
    /// Gateway node reward = (selfpoint / totalpoints) * total_reward
    /// input:
    ///  -total_reward: u128
    ///  -index: EraIndex
    fn compute_gateways_reward(total_reward: u128, index: EraIndex) {
        let total_points = CurrentTotalPoints::<T>::get();
        // Get the gateway node and its points
        let gateway_points = GatewayNodePoints::<T>::iter();

        for (who, point) in gateway_points {
            // Calculate the rate of gateway rewards
            let ratio = Perbill::from_rational(point, total_points);
            // Calculate the reward of gateway node
            let reward =
                T::BalanceToNumber::convert(ratio * T::NumberToBalance::convert(total_reward));
            // Save the gateway reward information
            T::MarketInterface::save_gateway_reward(who.clone(), reward, index);
        }
    }

    /// clear_points_info
    /// When the current era's award is calculated, the individual's gateway node poinst
    /// and total_reward need to be reset.Then save the total points in 'CurrentTotalPoints'
    /// input:
    ///     -index: EraIndex
    fn clear_points_info(index: EraIndex) {
        let current_total_points = CurrentTotalPoints::<T>::get();
        Self::deposit_event(Event::CurrentTotalPoints(current_total_points));
        EraTotalPoints::<T>::insert(index, current_total_points);
        CurrentTotalPoints::<T>::set(0);
        // todo
        // Get the online gateway node and their points
        let gateway_node_points = GatewayNodePoints::<T>::iter();
        // TODO remove all
        for (who, _) in gateway_node_points {
            // Reset the gateway node points = remve the points information
            GatewayNodePoints::<T>::remove(who.clone());
        }

        // Send the Event: ClearPoinstSuccess
        Self::deposit_event(Event::ClearPoinstSuccess);
    }

    fn clear_gateway_info(who: <T as frame_system::Config>::AccountId, peer_id: Vec<u8>) {
        // 0. get the peer_id list from Gateways
        let mut peer_id_list = Gateways::<T>::get();

        if peer_id_list.contains(&peer_id) {
            // 1. get the peer_id's index in the peer_id_list
            let mut index = 0;
            for p in &peer_id_list {
                if p.eq(&peer_id) {
                    break;
                }
                index += 1;
            }

            // 2. remove the peer_id from peer_id_list
            peer_id_list.remove(index);

            // 3. reset the list
            Gateways::<T>::put(peer_id_list);
        }

        // 5. reset the gateway num
        let mut gateway_num = GatewayNodeCount::<T>::get();
        gateway_num -= 1;
        GatewayNodeCount::<T>::set(gateway_num);

        // 6. remove the map account and peerid
        let mut account_peer_list = AccountPeerMap::<T>::get(who.clone()).unwrap();

        // 7. get the peer index
        let mut index = 0;
        for p in &account_peer_list {
            if p.eq(&peer_id) {
                break;
            }
            index += 1;
        }

        // 8. remove the peer id in the map account
        account_peer_list.remove(index);

        // 9. update
        AccountPeerMap::<T>::insert(who.clone(), account_peer_list);

        Self::deposit_event(Event::ClearGatewayInfoSuccess(who.clone(), peer_id.clone()));
    }
    // TODO rename
    fn accont_own_peerid(who: <T as frame_system::Config>::AccountId, peerid: Vec<u8>) -> bool {
        if !AccountPeerMap::<T>::contains_key(who.clone()) {
            return false;
        }

        let peer_list = AccountPeerMap::<T>::get(who.clone()).unwrap();

        // determine the peerid exit int the peer_list
        if !peer_list.contains(&peerid) {
            return false;
        }

        return true;
    }

    fn gateway_online_list() -> (Vec<(<T as frame_system::Config>::AccountId, Vec<Vec<u8>>)>, usize) {
        (AccountPeerMap::<T>::iter()
            .map(|(who, peer_list)| (who.clone(), peer_list.clone()))
            .collect(),
            Gateways::<T>::get().len())
    }
}

// Determine whether we run the storage migration logic
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
pub enum Releases {
    V1_0_0,
    V2_0_0,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V1_0_0
    }
}

pub mod migrations {

    pub mod v2 {}
}
