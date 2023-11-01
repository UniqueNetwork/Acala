#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::tokens::nonfungibles_v2::{Create, Inspect, Mutate, Transfer},
};
use frame_system::pallet_prelude::*;
use frame_system::Config as SystemConfig;
use sp_runtime::{traits::AccountIdConversion, DispatchError, DispatchResult};
use sp_std::{boxed::Box, vec::Vec};
use xcm::v3::{
	AssetId, AssetInstance, Fungibility, Junction::*, Junctions::*, MultiAsset, MultiLocation, Result as XcmResult,
};
use xcm_executor::traits::{ConvertLocation, Error as XcmExecutorError, TransactAsset};

pub mod impl_matches;
pub mod impl_nonfungibles;
pub mod impl_transactor;
pub mod types;
pub mod xcm_helpers;
pub use pallet::*;
pub(crate) use types::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + module_nft::Config
	where
		ItemIdOf<Self>: MaxEncodedLen,
		CollectionIdOf<Self>: MaxEncodedLen,
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type AccountIdConverter: ConvertLocation<Self::AccountId>;

		type NtfPalletLocation: Get<MultiLocation>;
	}

	/// Error for non-fungible-token module.
	#[pallet::error]
	pub enum Error<T> {
		AssetAlreadyRegistered,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config>
	where
		ItemIdOf<T>: MaxEncodedLen,
		CollectionIdOf<T>: MaxEncodedLen,
	{
		RegisteredAsset {
			asset_id: AssetId,
			collection_id: CollectionIdOf<T>,
		},
	}

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub type AssetsMapping<T: Config> = StorageMap<_, Twox64Concat, AssetId, CollectionIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn items)]
	pub type ItemsMapping<T: Config> =
		StorageDoubleMap<_, Twox64Concat, CollectionIdOf<T>, Twox64Concat, AssetInstance, ItemIdOf<T>, OptionQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		ItemIdOf<T>: MaxEncodedLen + Default,
		CollectionIdOf<T>: MaxEncodedLen,
	{
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn register_asset(origin: OriginFor<T>, foreign_asset: Box<AssetId>) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(
				!<AssetsMapping<T>>::contains_key(foreign_asset.as_ref()),
				<Error<T>>::AssetAlreadyRegistered,
			);
			let collection_id = module_nft::Pallet::<T>::create_collection(
				&Self::account_id(),
				&Self::account_id(),
				&Default::default(),
			)?;
			<AssetsMapping<T>>::insert(foreign_asset.as_ref(), collection_id.clone());
			Self::deposit_event(Event::RegisteredAsset {
				asset_id: *foreign_asset,
				collection_id,
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T>
where
	ItemIdOf<T>: MaxEncodedLen + Default,
	CollectionIdOf<T>: MaxEncodedLen,
{
	pub fn account_id() -> T::AccountId {
		frame_support::PalletId(*b"poc_xnft").into_account_truncating()
	}
}
