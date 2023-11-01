use module_nft::{ClassIdOf, TokenIdOf};

use crate::*;

pub type ItemIdOf<T> = TokenIdOf<T>;

pub type CollectionIdOf<T> = ClassIdOf<T>;

pub type ConverterOf<T> = <T as Config>::AccountIdConverter;

pub type ModuleNftPallet<T> = module_nft::Pallet<T>;

pub type OrmlNftPallet<T> = orml_nft::Pallet<T>;
