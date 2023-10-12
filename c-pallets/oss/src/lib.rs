#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_system::pallet_prelude::*;
use frame_support::{
	pallet_prelude::*, transactional,
};
use cp_cess_common::*;

pub use pallet::*;

pub use weights::WeightInfo;

type AccountOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use crate::*;
	use frame_system::ensure_signed;

	#[pallet::config]
	pub trait Config: frame_system::Config + sp_std::fmt::Debug {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type P2PLength: Get<u32> + Clone;

		#[pallet::constant]
		type AuthorLimit: Get<u32> + Clone;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//Successful Authorization Events
		Authorize { acc: AccountOf<T>, operator: AccountOf<T> },
		//Cancel authorization success event
		CancelAuthorize { acc: AccountOf<T>, oss: AccountOf<T> },
		//The event of successful Oss registration
		OssRegister { acc: AccountOf<T>, endpoint: PeerId },
		//Oss information change success event
		OssUpdate { acc: AccountOf<T>, new_endpoint: PeerId },
		//Oss account destruction success event
		OssDestroy { acc: AccountOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		//No errors authorizing any use
		NoAuthorization,
		//Registered Error
		Registered,
		//Unregistered Error
		UnRegister,
		//Option parse Error
		OptionParseError,

		BoundedVecError,

		Existed,
	}

	#[pallet::storage]
	#[pallet::getter(fn authority_list)]
	pub(super) type AuthorityList<T: Config> = StorageMap<_, Blake2_128Concat, AccountOf<T>, BoundedVec<AccountOf<T>, T::AuthorLimit>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn oss)]
	pub(super) type Oss<T: Config> = StorageMap<_, Blake2_128Concat, AccountOf<T>, PeerId>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[transactional]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::authorize())]
		pub fn authorize(origin: OriginFor<T>, operator: AccountOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			AuthorityList::<T>::try_mutate(&sender, |authority_list| -> DispatchResult {
				if !authority_list.contains(&operator) {
					authority_list.try_push(operator.clone()).map_err(|_| Error::<T>::BoundedVecError)?;
				}

				Ok(())
			})?;

			Self::deposit_event(Event::<T>::Authorize {
				acc: sender,
				operator,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[transactional]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::cancel_authorize())]
		pub fn cancel_authorize(origin: OriginFor<T>, oss: AccountOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<AuthorityList<T>>::contains_key(&sender), Error::<T>::NoAuthorization);

			AuthorityList::<T>::try_mutate(&sender, |authority_list| -> DispatchResult {
				authority_list.retain(|authority| authority != &oss); 

				Ok(())
			})?;

			Self::deposit_event(Event::<T>::CancelAuthorize {
				acc: sender,
				oss,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[transactional]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register())]
		pub fn register(origin: OriginFor<T>, endpoint: PeerId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(!<Oss<T>>::contains_key(&sender), Error::<T>::Registered);
			<Oss<T>>::insert(&sender, endpoint.clone());

			Self::deposit_event(Event::<T>::OssRegister {acc: sender, endpoint});

			Ok(())
		}

		#[pallet::call_index(3)]
		#[transactional]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::update())]
		pub fn update(origin: OriginFor<T>, endpoint: PeerId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<Oss<T>>::contains_key(&sender), Error::<T>::UnRegister);

			<Oss<T>>::try_mutate(&sender, |endpoint_opt| -> DispatchResult {
				let p_endpoint = endpoint_opt.as_mut().ok_or(Error::<T>::OptionParseError)?;
				*p_endpoint = endpoint.clone();
				Ok(())
			})?;

			Self::deposit_event(Event::<T>::OssUpdate {acc: sender, new_endpoint: endpoint});

			Ok(())
		}

		#[pallet::call_index(4)]
		#[transactional]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::destroy())]
		pub fn destroy(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<Oss<T>>::contains_key(&sender), Error::<T>::UnRegister);

			<Oss<T>>::remove(&sender);

			Self::deposit_event(Event::<T>::OssDestroy { acc: sender });

			Ok(())
		}
	}
}

pub trait OssFindAuthor<AccountId> {
	fn is_authorized(owner: AccountId, operator: AccountId) -> bool;
}

impl<T: Config> OssFindAuthor<AccountOf<T>> for Pallet<T> {
	fn is_authorized(owner: AccountOf<T>, operator: AccountOf<T>) -> bool {
		let acc_list = <AuthorityList<T>>::get(&owner);
		acc_list.contains(&operator)
	}
}