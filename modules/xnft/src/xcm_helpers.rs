use crate::*;
use xcm::v3::AssetId::Concrete;
use xcm_executor::traits::Error as MatchError;

impl<T: Config> Pallet<T>
where
	TokenIdOf<T>: TryFrom<u128>,
	ClassIdOf<T>: TryFrom<u128>,
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
		if asset_location.parents != T::NtfPalletLocation::get().parents {
			return None;
		}
		match asset_location
			.interior
			.match_and_split(T::NtfPalletLocation::get().interior())
		{
			Some(GeneralIndex(index)) => {
				let class_id = (*index).try_into().ok()?;
				Self::classes(class_id).is_none().then_some(class_id)
			}
			_ => None,
		}
	}

	pub fn deposit_foreign_asset(to: &T::AccountId, asset: ClassIdOf<T>, asset_instance: &AssetInstance) -> XcmResult {
		match Self::items(asset, asset_instance) {
			Some(token_id) => <ModuleNftPallet<T>>::do_transfer(&Self::account_id(), to, (asset, token_id))
				.map_err(|_| XcmError::FailedToTransactAsset("non-fungible foreign item deposit failed")),
			None => {
				let token_id = <OrmlNftPallet<T>>::mint(to, asset, Default::default(), Default::default())
					.map_err(|_| XcmError::FailedToTransactAsset("non-fungible new foreign item deposit failed"))?;
				<ItemsMapping<T>>::insert(asset, asset_instance, token_id);
				Ok(())
			}
		}
	}

	pub fn deposit_local_asset(to: &T::AccountId, asset: ClassIdOf<T>, asset_instance: &AssetInstance) -> XcmResult {
		let token_id = Self::convert_asset_instance(asset_instance)?;
		<ModuleNftPallet<T>>::do_transfer(&Self::account_id(), to, (asset, token_id))
			.map_err(|_| XcmError::FailedToTransactAsset("non-fungible local item deposit failed"))
	}

	pub fn asset_instance_to_token_id(
		class_id: ClassIdOf<T>,
		is_foreign_asset: bool,
		asset_instance: &AssetInstance,
	) -> Option<TokenIdOf<T>> {
		match is_foreign_asset {
			true => Self::items(class_id, asset_instance),
			false => Self::convert_asset_instance(asset_instance).ok(),
		}
	}
	pub fn convert_asset_instance(asset: &AssetInstance) -> Result<TokenIdOf<T>, MatchError> {
		let AssetInstance::Index(index) = asset else {
			return Err(MatchError::InstanceConversionFailed);
		};

		(*index).try_into().map_err(|_| MatchError::InstanceConversionFailed)
	}
}
