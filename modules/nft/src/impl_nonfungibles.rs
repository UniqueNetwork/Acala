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
use crate::*;

use frame_support::traits::tokens::nonfungibles_v2::{Create, Inspect, Mutate, Transfer};
use orml_nft::Pallet as OrmlPallet;

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
	type ItemId = TokenIdOf<T>;

	type CollectionId = ClassIdOf<T>;

	fn owner(collection: &Self::CollectionId, item: &Self::ItemId) -> Option<T::AccountId> {
		orml_nft::Pallet::<T>::tokens(collection, item).map(|d| d.owner)
	}

	fn collection_owner(collection: &Self::CollectionId) -> Option<T::AccountId> {
		orml_nft::Pallet::<T>::classes(collection).map(|d| d.owner)
	}

	fn attribute(_collection: &Self::CollectionId, _item: &Self::ItemId, _key: &[u8]) -> Option<Vec<u8>> {
		None
	}

	fn typed_attribute<K: Encode, V: Decode>(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		key: &K,
	) -> Option<V> {
		key.using_encoded(|d| Self::attribute(collection, item, d))
			.and_then(|v| V::decode(&mut &v[..]).ok())
	}

	fn collection_attribute(_collection: &Self::CollectionId, _key: &[u8]) -> Option<Vec<u8>> {
		None
	}

	fn typed_collection_attribute<K: Encode, V: Decode>(collection: &Self::CollectionId, key: &K) -> Option<V> {
		key.using_encoded(|d| Self::collection_attribute(collection, d))
			.and_then(|v| V::decode(&mut &v[..]).ok())
	}

	fn can_transfer(collection: &Self::CollectionId, _item: &Self::ItemId) -> bool {
		orml_nft::Pallet::<T>::classes(collection)
			.map(|i| i.data.properties.0.contains(ClassProperty::Transferable))
			.unwrap_or_default()
	}
}

impl<T: Config> Create<T::AccountId, Properties> for Pallet<T> {
	fn create_collection(
		who: &T::AccountId,
		_admin: &T::AccountId,
		_config: &Properties,
	) -> Result<Self::CollectionId, DispatchError> {
		let properties = Properties(ClassProperty::Mintable | ClassProperty::Burnable | ClassProperty::Transferable);
		let data = ClassData {
			deposit: Default::default(),
			properties,
			attributes: Default::default(),
		};
		orml_nft::Pallet::<T>::create_class(who, Default::default(), data)
	}
}

impl<T: Config> Mutate<T::AccountId, TokenData<BalanceOf<T>>> for Pallet<T> {
	fn mint_into(
		collection: &Self::CollectionId,
		_item: &Self::ItemId,
		who: &T::AccountId,
		config: &TokenData<BalanceOf<T>>,
		_deposit_collection_owner: bool,
	) -> frame_support::pallet_prelude::DispatchResult {
		orml_nft::Pallet::<T>::mint(who, *collection, Default::default(), config.clone()).map(|_| ())
	}

	fn burn(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		maybe_check_owner: Option<&T::AccountId>,
	) -> frame_support::pallet_prelude::DispatchResult {
		orml_nft::Pallet::<T>::burn(
			maybe_check_owner.unwrap_or(&Self::owner(collection, item).ok_or("owner not found")?),
			(*collection, *item),
		)
	}

	fn set_attribute(
		_collection: &Self::CollectionId,
		_item: &Self::ItemId,
		_key: &[u8],
		_value: &[u8],
	) -> frame_support::pallet_prelude::DispatchResult {
		Err(sp_runtime::TokenError::Unsupported.into())
	}

	fn set_typed_attribute<K: Encode, V: Encode>(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		key: &K,
		value: &V,
	) -> frame_support::pallet_prelude::DispatchResult {
		key.using_encoded(|k| value.using_encoded(|v| Self::set_attribute(collection, item, k, v)))
	}

	fn set_collection_attribute(
		_collection: &Self::CollectionId,
		_key: &[u8],
		_value: &[u8],
	) -> frame_support::pallet_prelude::DispatchResult {
		Err(sp_runtime::TokenError::Unsupported.into())
	}

	fn set_typed_collection_attribute<K: Encode, V: Encode>(
		collection: &Self::CollectionId,
		key: &K,
		value: &V,
	) -> frame_support::pallet_prelude::DispatchResult {
		key.using_encoded(|k| value.using_encoded(|v| Self::set_collection_attribute(collection, k, v)))
	}

	fn clear_attribute(
		_collection: &Self::CollectionId,
		_item: &Self::ItemId,
		_key: &[u8],
	) -> frame_support::pallet_prelude::DispatchResult {
		Err(sp_runtime::TokenError::Unsupported.into())
	}

	fn clear_typed_attribute<K: Encode>(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		key: &K,
	) -> frame_support::pallet_prelude::DispatchResult {
		key.using_encoded(|k| Self::clear_attribute(collection, item, k))
	}

	fn clear_collection_attribute(
		_collection: &Self::CollectionId,
		_key: &[u8],
	) -> frame_support::pallet_prelude::DispatchResult {
		Err(sp_runtime::TokenError::Unsupported.into())
	}

	fn clear_typed_collection_attribute<K: Encode>(
		collection: &Self::CollectionId,
		key: &K,
	) -> frame_support::pallet_prelude::DispatchResult {
		key.using_encoded(|k| Self::clear_collection_attribute(collection, k))
	}
}
impl<T: Config> Transfer<T::AccountId> for Pallet<T> {
	fn transfer(
		collection: &Self::CollectionId,
		item: &Self::ItemId,
		destination: &T::AccountId,
	) -> frame_support::pallet_prelude::DispatchResult {
		let from = &Self::owner(collection, item).ok_or("owner not found")?;
		<OrmlPallet<T>>::transfer(from, destination, (*collection, *item))
	}
}
