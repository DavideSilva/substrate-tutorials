#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::storage]
	#[pallet::getter(fn event_counter)]
	pub type EventCounter<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn reminders)]
	pub type Reminders<T: Config> =
		StorageMap<_, Blake2_256, T::BlockNumber, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ReminderSet(T::BlockNumber, Vec<u8>),
		Reminder(Vec<u8>),
		RemindersExecuteds(u32),
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let mut used_weight = 0;
			let reminders = Self::reminders(n);

			used_weight += T::DbWeight::get().reads(1);
			used_weight += T::DbWeight::get().writes(2);

			let mut events = 0;

			for reminder in reminders {
				Self::deposit_event(Event::Reminder(reminder.clone()));
				events += 1;
			}

			Reminders::<T>::remove(n);

			EventCounter::<T>::mutate(|event_counter| *event_counter = events);

			used_weight
		}

		fn on_finalize(_: T::BlockNumber) {
			Self::deposit_event(Event::RemindersExecuteds(Self::event_counter()));
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads(1))]
		pub fn schedule_reminder(
			origin: OriginFor<T>,
			at: T::BlockNumber,
			message: Vec<u8>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			<Reminders<T>>::mutate(at, |reminders| reminders.push(message.clone()));
			Self::deposit_event(Event::ReminderSet(at, message));

			Ok(())
		}
	}
}
