// This file is part of Acala.

// Copyright (C) 2020-2023 Acala Foundation.
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

//! Unit tests for nominees election module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn bond_below_min_bond_threshold() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 4),
			Error::<Runtime>::BelowMinBondThreshold,
		);
	});
}

#[test]
fn bond_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 50));
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).frozen, 50);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 50);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().active(), 50);
	});
}

#[test]
fn bond_amount_over_remain_free() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 2000));
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).frozen, 1000);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 1000);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().active(), 1000);
	});
}

#[test]
fn unbond_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 200));
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 200);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().active(), 100);
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).frozen, 200);
		NomineesElectionModule::on_new_era(4);
		assert_ok!(NomineesElectionModule::withdraw_unbonded(RuntimeOrigin::signed(ALICE)));
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 100);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().active(), 100);
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).frozen, 100);
	});
}

#[test]
fn unbond_exceed_max_unlock_chunk() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 1000));
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		NomineesElectionModule::on_new_era(1);
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		NomineesElectionModule::on_new_era(2);
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		NomineesElectionModule::on_new_era(3);
		assert_noop!(
			NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100),
			Error::<Runtime>::MaxUnlockChunksExceeded,
		);
	});
}

#[test]
fn unbond_amount_over_active() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 1000));
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 1500));
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 1000);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().active(), 0);
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).frozen, 1000);
		NomineesElectionModule::on_new_era(4);
		assert_ok!(NomineesElectionModule::withdraw_unbonded(RuntimeOrigin::signed(ALICE)));
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).frozen, 0);
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).free, 1000);
	});
}

#[test]
fn unbond_remain_below_threshold() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 1000));
		assert_noop!(
			NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 996),
			Error::<Runtime>::BelowMinBondThreshold,
		);
	});
}

#[test]
fn rebond_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			NomineesElectionModule::rebond(RuntimeOrigin::signed(ALICE), 100),
			Error::<Runtime>::NotBonded,
		);
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 1000));
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		NomineesElectionModule::on_new_era(1);
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		NomineesElectionModule::on_new_era(2);
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		NomineesElectionModule::on_new_era(3);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 1000);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().active(), 700);
		assert_ok!(NomineesElectionModule::rebond(RuntimeOrigin::signed(ALICE), 150));
		System::assert_last_event(mock::RuntimeEvent::NomineesElectionModule(crate::Event::Rebond {
			who: ALICE,
			amount: 150,
		}));
		NomineesElectionModule::on_new_era(4);
		assert_ok!(NomineesElectionModule::withdraw_unbonded(RuntimeOrigin::signed(ALICE)));
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 900);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().active(), 850);
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).frozen, 900);

		assert_ok!(NomineesElectionModule::rebond(RuntimeOrigin::signed(ALICE), 200));
		System::assert_last_event(mock::RuntimeEvent::NomineesElectionModule(crate::Event::Rebond {
			who: ALICE,
			amount: 50,
		}));
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 900);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().active(), 900);
		assert_eq!(TokensModule::accounts(&ALICE, LDOT).frozen, 900);
	});
}

#[test]
fn withdraw_unbonded_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(NomineesElectionModule::current_era(), 0);
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 1000));
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 1000);
		NomineesElectionModule::on_new_era(3);
		assert_ok!(NomineesElectionModule::withdraw_unbonded(RuntimeOrigin::signed(ALICE)));
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 1000);
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().unlocking_len(), 1);
		assert_ok!(NomineesElectionModule::unbond(RuntimeOrigin::signed(ALICE), 100));
		NomineesElectionModule::on_new_era(4);
		assert_ok!(NomineesElectionModule::withdraw_unbonded(RuntimeOrigin::signed(ALICE)));
		assert_eq!(NomineesElectionModule::ledger(&ALICE).unwrap().total(), 900);
	});
}

#[test]
fn nominate_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NomineesElectionModule::nominate(RuntimeOrigin::signed(ALICE), vec![1, 2, 3, 4, 5]),
			Error::<Runtime>::NotBonded,
		);

		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 500));

		assert_noop!(
			NomineesElectionModule::nominate(RuntimeOrigin::signed(ALICE), vec![]),
			Error::<Runtime>::InvalidTargetsLength,
		);
		assert_noop!(
			NomineesElectionModule::nominate(RuntimeOrigin::signed(ALICE), vec![1, 2, 3, 4, 5, 6]),
			Error::<Runtime>::InvalidTargetsLength,
		);

		assert_eq!(NomineesElectionModule::nominations(&ALICE), vec![]);
		assert_eq!(NomineesElectionModule::votes(1), 0);
		assert_ok!(NomineesElectionModule::nominate(
			RuntimeOrigin::signed(ALICE),
			vec![1, 2, 3, 4, 5]
		));
		assert_eq!(NomineesElectionModule::nominations(&ALICE), vec![1, 2, 3, 4, 5]);
		assert_eq!(NomineesElectionModule::votes(1), 500);
		assert_eq!(NomineesElectionModule::votes(2), 500);
		assert_ok!(NomineesElectionModule::nominate(
			RuntimeOrigin::signed(ALICE),
			vec![2, 3, 4, 5, 6]
		));
		assert_eq!(NomineesElectionModule::nominations(&ALICE), vec![2, 3, 4, 5, 6]);
		assert_eq!(NomineesElectionModule::votes(1), 0);
		assert_eq!(NomineesElectionModule::votes(2), 500);
	});
}

#[test]
fn chill_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 500));
		assert_ok!(NomineesElectionModule::nominate(
			RuntimeOrigin::signed(ALICE),
			vec![1, 2, 3, 4, 5]
		));
		assert_eq!(NomineesElectionModule::nominations(&ALICE), vec![1, 2, 3, 4, 5]);
		assert_eq!(NomineesElectionModule::votes(1), 500);
		assert_eq!(NomineesElectionModule::votes(2), 500);
		assert_ok!(NomineesElectionModule::chill(RuntimeOrigin::signed(ALICE)));
		assert_eq!(NomineesElectionModule::nominations(&ALICE), vec![]);
		assert_eq!(NomineesElectionModule::votes(1), 0);
		assert_eq!(NomineesElectionModule::votes(2), 0);
	});
}

#[test]
fn rebalance_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(ALICE), 500));
		assert_ok!(NomineesElectionModule::nominate(
			RuntimeOrigin::signed(ALICE),
			vec![1, 2, 3, 4, 5]
		));
		assert_eq!(NomineesElectionModule::nominees(), vec![]);
		assert_eq!(NomineesElectionModule::nominees().len(), 0);
		NomineesElectionModule::rebalance();
		assert_eq!(NomineesElectionModule::nominees().len(), 5);
		assert!(NomineesElectionModule::nominees().contains(&1));
		assert_ok!(NomineesElectionModule::bond(RuntimeOrigin::signed(BOB), 600));
		assert_ok!(NomineesElectionModule::nominate(
			RuntimeOrigin::signed(ALICE),
			vec![2, 3, 4, 5, 6]
		));
		NomineesElectionModule::rebalance();
		assert_eq!(NomineesElectionModule::nominees().len(), 5);
		assert!(!NomineesElectionModule::nominees().contains(&1));
	});
}

#[test]
fn update_votes_work() {
	ExtBuilder::default().build().execute_with(|| {
		<Votes<Runtime>>::insert(1, 50);
		<Votes<Runtime>>::insert(2, 100);
		NomineesElectionModule::update_votes(30, &[1, 2], 50, &[1, 2]);
		assert_eq!(NomineesElectionModule::votes(1), 70);
		assert_eq!(NomineesElectionModule::votes(2), 120);
		NomineesElectionModule::update_votes(0, &[1, 2], 50, &[3, 4]);
		assert_eq!(NomineesElectionModule::votes(1), 70);
		assert_eq!(NomineesElectionModule::votes(2), 120);
		assert_eq!(NomineesElectionModule::votes(3), 50);
		assert_eq!(NomineesElectionModule::votes(4), 50);
		NomineesElectionModule::update_votes(200, &[1, 2, 3, 4], 10, &[3, 4]);
		assert_eq!(NomineesElectionModule::votes(1), 0);
		assert_eq!(NomineesElectionModule::votes(2), 0);
		assert_eq!(NomineesElectionModule::votes(3), 10);
		assert_eq!(NomineesElectionModule::votes(4), 10);
	});
}
