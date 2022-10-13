// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_resource_order
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-09-09, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `gzhlinux`, CPU: `AMD Ryzen 7 5800H with Radeon Graphics`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/substrate
// benchmark
// pallet
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_resource_order
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --output
// ./frame/resource-order/src/weights.rs
// --template
// ./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_resource_order.
pub trait WeightInfo {
	fn create_order_info() -> Weight;
	fn order_exec() -> Weight;
	fn heartbeat() -> Weight;
	fn cancel_order() -> Weight;
	fn renew_agreement() -> Weight;
}

/// Weights for pallet_resource_order using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Market Staking (r:1 w:0)
	// Storage: Provider Resources (r:1 w:1)
	// Storage: ResourceOrder OrderIndex (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: ResourceOrder UserOrders (r:1 w:1)
	// Storage: ResourceOrder ResourceOrders (r:0 w:1)
	fn create_order_info() -> Weight {
		(38_133_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Market Staking (r:2 w:1)
	// Storage: ResourceOrder ResourceOrders (r:1 w:1)
	// Storage: Market ClientBaseFee (r:1 w:0)
	// Storage: Market TotalStaked (r:1 w:1)
	// Storage: Provider Resources (r:1 w:1)
	// Storage: ResourceOrder AgreementIndex (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: ResourceOrder BlockWithAgreement (r:1 w:1)
	// Storage: ResourceOrder UserAgreements (r:1 w:1)
	// Storage: ResourceOrder ProviderAgreements (r:1 w:1)
	// Storage: ResourceOrder RentalAgreements (r:0 w:1)
	fn order_exec() -> Weight {
		(56_151_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(11 as u64))
			.saturating_add(T::DbWeight::get().writes(9 as u64))
	}
	// Storage: ResourceOrder RentalAgreements (r:1 w:1)
	fn heartbeat() -> Weight {
		(24_793_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ResourceOrder ResourceOrders (r:1 w:1)
	// Storage: Provider Resources (r:1 w:1)
	fn cancel_order() -> Weight {
		(29_612_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: ResourceOrder RentalAgreements (r:1 w:0)
	// Storage: Provider Resources (r:1 w:0)
	// Storage: ResourceOrder OrderIndex (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: ResourceOrder UserOrders (r:1 w:1)
	// Storage: ResourceOrder ResourceOrders (r:0 w:1)
	fn renew_agreement() -> Weight {
		(47_142_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Market Staking (r:1 w:0)
	// Storage: Provider Resources (r:1 w:1)
	// Storage: ResourceOrder OrderIndex (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: ResourceOrder UserOrders (r:1 w:1)
	// Storage: ResourceOrder ResourceOrders (r:0 w:1)
	fn create_order_info() -> Weight {
		(38_133_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Market Staking (r:2 w:1)
	// Storage: ResourceOrder ResourceOrders (r:1 w:1)
	// Storage: Market ClientBaseFee (r:1 w:0)
	// Storage: Market TotalStaked (r:1 w:1)
	// Storage: Provider Resources (r:1 w:1)
	// Storage: ResourceOrder AgreementIndex (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: ResourceOrder BlockWithAgreement (r:1 w:1)
	// Storage: ResourceOrder UserAgreements (r:1 w:1)
	// Storage: ResourceOrder ProviderAgreements (r:1 w:1)
	// Storage: ResourceOrder RentalAgreements (r:0 w:1)
	fn order_exec() -> Weight {
		(56_151_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(11 as u64))
			.saturating_add(RocksDbWeight::get().writes(9 as u64))
	}
	// Storage: ResourceOrder RentalAgreements (r:1 w:1)
	fn heartbeat() -> Weight {
		(24_793_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: ResourceOrder ResourceOrders (r:1 w:1)
	// Storage: Provider Resources (r:1 w:1)
	fn cancel_order() -> Weight {
		(29_612_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: ResourceOrder RentalAgreements (r:1 w:0)
	// Storage: Provider Resources (r:1 w:0)
	// Storage: ResourceOrder OrderIndex (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: ResourceOrder UserOrders (r:1 w:1)
	// Storage: ResourceOrder ResourceOrders (r:0 w:1)
	fn renew_agreement() -> Weight {
		(47_142_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
}