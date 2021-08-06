#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use xcm::v0::{ Junction, OriginKind, SendXcm, Xcm,};
	use cumulus_primitives_core::ParaId;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
	pub struct XregisterCall<AccountId> {
		call_index: [u8; 2],
		account: AccountId,
		name: Vec<u8>,
	}

	impl<AccountId> XregisterCall<AccountId> {
		pub fn new(pallet_index: u8, method_index: u8, account: AccountId, name: Vec<u8>,) 
		-> Self {
			XregisterCall {
				call_index: [pallet_index, method_index],
				account: account,
				name: name,
			}
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The XCM sender module.
		type XcmSender: SendXcm;

		/// Xregister server's parachain ID
		type XregisterServerParachainId: Get<ParaId>;

		/// Xregister Pallet ID in xregister server
		type XregisterPalletID: Get<u8>;

		/// Xregister Method ID in xregister server
		type XregisterMethodID: Get<u8>;

		/// Xregister maximum weight
		type XregisterWeightAtMost: Get<u64>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Xregister(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error to send xcm to Xregister server
		XcmSendError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn xregister(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// compose the call with pallet id, method id and arguments
			let call = XregisterCall::<T::AccountId>::new(
				T::XregisterPalletID::get(), 
				T::XregisterMethodID::get(),
				who.clone(),
				name.clone());
	
			// build the xcm transact message
			let message = Xcm::Transact { 
				origin_type: OriginKind::Native, 
				require_weight_at_most: T::XregisterWeightAtMost::get(), 
				call: call.encode().into() };
			
			// send the message to xregister server chain
			match T::XcmSender::send_xcm((Junction::Parent, 
					Junction::Parachain(T::XregisterServerParachainId::get().into())).into(), 
					message) {
				Ok(()) => {
					// emit the event if send successfully
					Self::deposit_event(Event::Xregister(who, name));
					Ok(().into())
				},
				Err(_) => Err(Error::<T>::XcmSendError.into()),
			}
		}
	}
}
