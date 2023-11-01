use crate::*;
use frame_support::traits::Incrementable;
use orml_nft::Tokens;
use xcm::v3::{AssetId::Concrete, Error as XcmError};
use xcm_executor::traits::{Error as MatchError, MatchesNonFungibles};

impl<T: Config> Pallet<T>
where
	TokenIdOf<T>: MaxEncodedLen + Incrementable + TryFrom<u128>,
	ClassIdOf<T>: MaxEncodedLen + TryFrom<u128>,
{
	pub fn asset_to_collection(asset: &AssetId) -> Result<(ClassIdOf<T>, bool), MatchError> {
		Self::foreign_asset_to_collection(asset)
			.map(|a| (a, true))
			.or_else(|| Self::local_asset_to_collection(asset).map(|a| (a, false)))
			.ok_or(MatchError::AssetIdConversionFailed)
	}

	pub fn foreign_asset_to_collection(asset: &AssetId) -> Option<ClassIdOf<T>> {
		Self::assets(asset)
	}

	pub fn local_asset_to_collection(asset: &AssetId) -> Option<ClassIdOf<T>> {
		let Concrete(asset_location) = asset else {
			return None;
		};

		match asset_location
			.interior
			.match_and_split(T::NtfPalletLocation::get().interior())
		{
			Some(GeneralIndex(index)) => Some((*index).try_into().ok()?),
			_ => None,
		}
	}

	pub fn deposit_foreign_asset(to: &T::AccountId, asset: ClassIdOf<T>, asset_instance: &AssetInstance) -> XcmResult {
		match Self::items(asset, asset_instance) {
			Some(token_id) => {
				let current_owner =
					<ModuleNftPallet<T>>::owner(&asset, &token_id).ok_or(MatchError::InstanceConversionFailed)?;
				<ModuleNftPallet<T>>::do_transfer(&current_owner, to, (asset, token_id))
					.map_err(|_| XcmError::FailedToTransactAsset("nonfungible item withdraw failed"))
			}
			None => {
				let token_id = <OrmlNftPallet<T>>::mint(to, asset, Default::default(), Default::default())
					.map_err(|_| XcmExecutorError::InstanceConversionFailed)?;
				<ItemsMapping<T>>::insert(asset, asset_instance, token_id);
				Ok(())
			}
		}
	}

	pub fn deposit_local_asset(to: &T::AccountId, asset: ClassIdOf<T>, asset_instance: &AssetInstance) -> XcmResult {
		let token_id = Self::convert_asset_instance(asset_instance)?;

		let current_owner =
			<ModuleNftPallet<T>>::owner(&asset, &token_id).ok_or(MatchError::InstanceConversionFailed)?;
		<ModuleNftPallet<T>>::do_transfer(&current_owner, to, (asset, token_id))
			.map_err(|_| XcmError::LocationNotInvertible)
	}

	pub fn convert_asset_instance(asset: &AssetInstance) -> Result<TokenIdOf<T>, MatchError> {
		let AssetInstance::Index(index) = asset else {
			return Err(MatchError::InstanceConversionFailed);
		};

		(*index).try_into().map_err(|_| MatchError::InstanceConversionFailed)
	}
}
