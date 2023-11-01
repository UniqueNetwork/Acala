use crate::*;
use frame_support::traits::Incrementable;

const LOG_TARGET: &str = "xcm::module_xnft::transactor";

impl<T: Config> TransactAsset for Pallet<T>
where
	TokenIdOf<T>: MaxEncodedLen + Incrementable + TryFrom<u128>,
	ClassIdOf<T>: MaxEncodedLen + TryFrom<u128>,
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

		let (collection_id, is_foreign_asset) = Self::asset_to_collection(&what.id)?;

		let to = <ConverterOf<T>>::convert_location(who).ok_or(XcmExecutorError::AccountIdConversionFailed)?;

		let deposit_handler = if is_foreign_asset {
			Self::deposit_foreign_asset
		} else {
			Self::deposit_local_asset
		};

		deposit_handler(&to, collection_id, &asset_instance)
	}

	fn withdraw_asset(
		_what: &MultiAsset,
		_who: &xcm::v3::MultiLocation,
		_maybe_context: Option<&xcm::v3::XcmContext>,
	) -> Result<xcm_executor::Assets, xcm::v3::Error> {
		Err(xcm::v3::Error::Unimplemented)
	}

	fn internal_transfer_asset(
		_asset: &MultiAsset,
		_from: &xcm::v3::MultiLocation,
		_to: &xcm::v3::MultiLocation,
		_context: &xcm::v3::XcmContext,
	) -> Result<xcm_executor::Assets, xcm::v3::Error> {
		Err(xcm::v3::Error::Unimplemented)
	}

	fn transfer_asset(
		asset: &MultiAsset,
		from: &xcm::v3::MultiLocation,
		to: &xcm::v3::MultiLocation,
		context: &xcm::v3::XcmContext,
	) -> Result<xcm_executor::Assets, xcm::v3::Error> {
		match Self::internal_transfer_asset(asset, from, to, context) {
			Err(xcm::v3::Error::AssetNotFound | xcm::v3::Error::Unimplemented) => {
				let assets = Self::withdraw_asset(asset, from, Some(context))?;
				// Not a very forgiving attitude; once we implement roll-backs then it'll be nicer.
				Self::deposit_asset(asset, to, context)?;
				Ok(assets)
			}
			result => result,
		}
	}
}
