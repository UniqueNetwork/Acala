use crate::*;

pub type ConverterOf<T> = <T as Config>::LocationToAccountId;

pub type ModuleNftPallet<T> = module_nft::Pallet<T>;

pub type OrmlNftPallet<T> = orml_nft::Pallet<T>;
