// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Tests parachain to parachain xcm communication between Statemine and Karura.

#[cfg(feature = "with-karura-runtime")]
mod statemine_tests {
	use crate::relaychain::kusama_test_net::*;
	use crate::setup::*;

	use frame_support::{assert_ok, traits::Hooks};
	use polkadot_parachain::primitives::Sibling;
	use xcm::{
		v1::{Junction, Junctions, MultiAsset, MultiAssets, MultiLocation},
		VersionedMultiAssets, VersionedMultiLocation,
	};
	use xcm_builder::AccountId32Aliases;
	use xcm_emulator::TestExt;
	use xcm_executor::traits::Convert;

	#[test]
	fn can_transfer_custom_asset_into_karura() {
		env_logger::init();
		Statemine::execute_with(|| {
			use westmint_runtime::*;

			let origin = Origin::signed(ALICE.into());
			Balances::make_free_balance_be(&ALICE.into(), 10 * dollar(KSM));

			// need to have some KSM to be able to receive user assets
			Balances::make_free_balance_be(&Sibling::from(2000).into_account(), 10 * dollar(KSM));

			assert_ok!(Assets::create(origin.clone(), 0, MultiAddress::Id(ALICE.into()), 10,));
			assert_ok!(Assets::mint(origin.clone(), 0, MultiAddress::Id(ALICE.into()), 1000));

			let para_acc: AccountId = Sibling::from(2000).into_account();

			// KSM is used to pay for xcm execution
			let multi_asset: VersionedMultiAssets = VersionedMultiAssets::V1(
				vec![
					MultiAsset {
						id: Concrete(MultiLocation::here()),
						fun: Fungibility::Fungible(dollar(KSM)),
					},
					MultiAsset {
						id: Concrete(MultiLocation {
							parents: 0,
							interior: Junctions::X1(Junction::GeneralIndex(0)),
						}),
						fun: Fungibility::Fungible(100),
					},
				]
				.into(),
			);
			// !todo : Figure out how to represent GeneralIndex(0) in MultiAsset so the transaction can go
			// through

			assert_ok!(PolkadotXcm::reserve_transfer_assets(
				origin.clone(),
				Box::new(MultiLocation::new(1, X1(Parachain(2000))).into()),
				Box::new(
					Junction::AccountId32 {
						id: BOB,
						network: NetworkId::Any
					}
					.into()
					.into()
				),
				Box::new(multi_asset),
				0
			));
			assert_eq!(Balances::free_balance(&ALICE.into()), 9 * dollar(KSM));
			assert_eq!(Assets::balance(0, &para_acc), 100);
		});

		// Rerun the Statemine::execute to actually send the egress message via XCM
		Statemine::execute_with(|| {
			let para_acc: AccountId = Sibling::from(2000).into_account();
			assert_eq!(westmint_runtime::Assets::balance(0, &para_acc), 100);
			println!("Westmint: {:?}", westmint_runtime::System::events());
		});

		Karura::execute_with(|| {
			println!("Karura: {:?}", karura_runtime::System::events());
			// assert_eq!(Tokens::free_balance(KSM, &AccountId::from(BOB)), 0);
		});
	}
}
