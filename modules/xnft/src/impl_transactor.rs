use crate::*;

const LOG_TARGET: &str = "xcm::module_xnft::transactor";

impl<T: Config> TransactAsset for Pallet<T>
where
	TokenIdOf<T>: TryFrom<u128>,
	ClassIdOf<T>: TryFrom<u128>,
{
	fn can_check_in(
		_origin: &xcm::v3::MultiLocation,
		_what: &MultiAsset,
		_context: &xcm::v3::XcmContext,
	) -> xcm::v3::Result {
		Err(xcm::v3::Error::Unimplemented)
	}

	fn check_in(_origin: &xcm::v3::MultiLocation, _what: &MultiAsset, _context: &xcm::v3::XcmContext) {}

	fn can_check_out(
		_dest: &xcm::v3::MultiLocation,
		_what: &MultiAsset,
		_context: &xcm::v3::XcmContext,
	) -> xcm::v3::Result {
		Err(xcm::v3::Error::Unimplemented)
	}

	fn check_out(_dest: &xcm::v3::MultiLocation, _what: &MultiAsset, _context: &xcm::v3::XcmContext) {}

	fn deposit_asset(what: &MultiAsset, who: &xcm::v3::MultiLocation, _context: &xcm::v3::XcmContext) -> XcmResult {
		log::trace!(
			target: LOG_TARGET,
			"deposit_asset what: {:?}, who: {:?}, context: {:?}",
			what,
			who,
			_context,
		);

		let Fungibility::NonFungible(asset_instance) = what.fun else {
			return Err(XcmExecutorError::AssetNotHandled.into());
		};

		let (class_id, is_foreign_asset) = Self::asset_to_collection(&what.id)?;

		let to = <ConverterOf<T>>::convert_location(who).ok_or(XcmExecutorError::AccountIdConversionFailed)?;

		let deposit_handler = if is_foreign_asset {
			Self::deposit_foreign_asset
		} else {
			Self::deposit_local_asset
		};

		deposit_handler(&to, class_id, &asset_instance)
	}

	fn withdraw_asset(
		what: &MultiAsset,
		who: &xcm::v3::MultiLocation,
		_maybe_context: Option<&xcm::v3::XcmContext>,
	) -> Result<xcm_executor::Assets, xcm::v3::Error> {
		log::trace!(
			target: LOG_TARGET,
			"withdraw_asset what: {:?}, who: {:?}, maybe_context: {:?}",
			what,
			who,
			_maybe_context,
		);

		let Fungibility::NonFungible(asset_instance) = what.fun else {
			return Err(XcmExecutorError::AssetNotHandled.into());
		};

		let (class_id, is_foreign_asset) = Self::asset_to_collection(&what.id)?;

		let from = <ConverterOf<T>>::convert_location(who).ok_or(XcmExecutorError::AccountIdConversionFailed)?;

		let token_id = Self::asset_instance_to_token_id(class_id, is_foreign_asset, &asset_instance)
			.ok_or(XcmExecutorError::InstanceConversionFailed)?;

		<ModuleNftPallet<T>>::do_transfer(&from, &Self::account_id(), (class_id, token_id))
			.map(|_| what.clone().into())
			.map_err(|_| XcmError::FailedToTransactAsset("non-fungible item withdraw failed"))
	}

	fn internal_transfer_asset(
		asset: &MultiAsset,
		from: &xcm::v3::MultiLocation,
		to: &xcm::v3::MultiLocation,
		_context: &xcm::v3::XcmContext,
	) -> Result<xcm_executor::Assets, xcm::v3::Error> {
		log::trace!(
			target: LOG_TARGET,
			"internal_transfer_asset: {:?}, from: {:?}, to: {:?}, context: {:?}",
			asset,
			from,
			to,
			_context
		);

		let Fungibility::NonFungible(asset_instance) = asset.fun else {
			return Err(XcmExecutorError::AssetNotHandled.into());
		};

		let (class_id, is_foreign_asset) = Self::asset_to_collection(&asset.id)?;

		let from = <ConverterOf<T>>::convert_location(from).ok_or(XcmExecutorError::AccountIdConversionFailed)?;
		let to = <ConverterOf<T>>::convert_location(to).ok_or(XcmExecutorError::AccountIdConversionFailed)?;

		let token_id = Self::asset_instance_to_token_id(class_id, is_foreign_asset, &asset_instance)
			.ok_or(XcmExecutorError::InstanceConversionFailed)?;

		<ModuleNftPallet<T>>::do_transfer(&from, &to, (class_id, token_id))
			.map(|_| asset.clone().into())
			.map_err(|_| XcmError::FailedToTransactAsset("non-fungible item internal transfer failed"))
	}
}
