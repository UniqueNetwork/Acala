//! # Homa Module
//!
//! ## Overview
//!
//! The user entrance of Homa protocol. User can inject DOT into the staking
//! pool and get LDOT, which is the redemption voucher for DOT owned by the
//! staking pool. The staking pool will staking these DOT to get staking
//! rewards. Holders of LDOT can choose different ways to redeem DOT.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_module, transactional, weights::Weight};
use frame_system::{self as system, ensure_signed};
use primitives::{Balance, EraIndex};
use sp_runtime::RuntimeDebug;
use support::HomaProtocol;

mod default_weight;

/// Redemption modes:
/// 1. Immediately: User will immediately get back DOT from the free pool, which
/// is a liquid pool operated by staking pool, but they have to pay extra fee.
/// 2. Target: User can claim the unclaimed unbonding DOT of specific era, after
/// the remaining unbinding period has passed, users can get back the DOT.
/// 3. WaitForUnbonding: User request unbond, the staking pool will process
/// unbonding in the next era, and user needs to wait for the complete unbonding
/// era which determined by Polkadot.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum RedeemStrategy {
	Immediately,
	Target(EraIndex),
	WaitForUnbonding,
}

pub trait WeightInfo {
	fn mint() -> Weight;
	fn redeem(strategy: &RedeemStrategy) -> Weight;
	fn withdraw_redemption() -> Weight;
}

pub trait Config: system::Config {
	/// The core of Homa protocol.
	type Homa: HomaProtocol<Self::AccountId, Balance, EraIndex>;

	/// Weight information for the extrinsics in this module.
	type WeightInfo: WeightInfo;
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		/// Inject DOT to staking pool and mint LDOT in a certain exchange rate decided by staking pool.
		///
		/// - `amount`: the DOT amount to inject into staking pool.
		#[weight = <T as Config>::WeightInfo::mint()]
		#[transactional]
		pub fn mint(origin, #[compact] amount: Balance) {
			let who = ensure_signed(origin)?;
			T::Homa::mint(&who, amount)?;
		}

		/// Burn LDOT and redeem DOT from staking pool.
		///
		/// - `amount`: the LDOT amount to redeem.
		/// - `strategy`: redemption mode.
		#[weight = <T as Config>::WeightInfo::redeem(strategy)]
		#[transactional]
		pub fn redeem(origin, #[compact] amount: Balance, strategy: RedeemStrategy) {
			let who = ensure_signed(origin)?;
			match strategy {
				RedeemStrategy::Immediately => {
					T::Homa::redeem_by_free_unbonded(&who, amount)?;
				},
				RedeemStrategy::Target(target_era) => {
					T::Homa::redeem_by_claim_unbonding(&who, amount, target_era)?;
				},
				RedeemStrategy::WaitForUnbonding => {
					T::Homa::redeem_by_unbond(&who, amount)?;
				},
			}
		}

		/// Get back those DOT that have been unbonded.
		#[weight = <T as Config>::WeightInfo::withdraw_redemption()]
		#[transactional]
		pub fn withdraw_redemption(origin) {
			let who = ensure_signed(origin)?;
			T::Homa::withdraw_redemption(&who)?;
		}
	}
}

impl<T: Config> Module<T> {}
