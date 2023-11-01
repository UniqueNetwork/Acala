// use crate::*;
// use frame_support::traits::Incrementable;
// use xcm::v3::Fungibility;
// use xcm_executor::traits::{Error as MatchError, MatchesNonFungibles};

// impl<T: Config> MatchesNonFungibles<CollectionIdOf<T>, ItemIdOf<T>> for Pallet<T>
// where
// 	ItemIdOf<T>: MaxEncodedLen + Incrementable,
// 	CollectionIdOf<T>: MaxEncodedLen,
// {
// 	fn matches_nonfungibles(
// 		foreign_asset: &MultiAsset,
// 	) -> core::result::Result<(CollectionIdOf<T>, ItemIdOf<T>), MatchError> {
// 		let Fungibility::NonFungible(asset_instance) = foreign_asset.fun else {
// 			return Err(MatchError::AssetNotHandled);
// 		};
// 		let asset = Self::assets(foreign_asset.id).ok_or(MatchError::AssetNotHandled)?;
// 		let item = Self::items(&asset, asset_instance).unwrap_or(Self::get_next_item_of(&asset)?);
// 		Ok((asset, item))
// 	}
// }
