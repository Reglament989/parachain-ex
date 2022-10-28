#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, parameter_types};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;

	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, RuntimeDebug)]
	#[scale_info(skip_type_params(T))]
	pub struct Post<T: Config> {
		id: <T as frame_system::Config>::Hash,
		title: Vec<u8>,
		body: Vec<u8>,
		author: <T as frame_system::Config>::AccountId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct PostComment<T: Config> {
		pub content: Vec<u8>,
		pub post_id: T::Hash,
		pub author: <T as frame_system::Config>::AccountId,
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + core::fmt::Debug {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type BlogPostMinBytes: Get<u32>;

		#[pallet::constant]
		type BlogPostMaxBytes: Get<u32>;

		#[pallet::constant]
		type BlogPostCommentMinBytes: Get<u32>;

		#[pallet::constant]
		type BlogPostCommentMaxBytes: Get<u32>;
	}

	parameter_types! {
		pub const BlogPostMinBytes: u32 = 64;
		pub const BlogPostMaxBytes: u32 = 4096;
		pub const BlogPostCommentMinBytes: u32 = 64;
		pub const BlogPostCommentMaxBytes: u32 = 1024;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn get_posts)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Blog<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Post<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_comments)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type BlogPostComments<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, Vec<PostComment<T>>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		PostStored(T::AccountId, T::Hash),
		CommentOfPostStored(T::AccountId, T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		BlogPostNotEnoughBytes,
		BlogPostTooManyBytes,
		BlogPostCommentNotEnoughBytes,
		BlogPostCommentTooManyBytes,
		BlogPostNotFound,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn post_blog(origin: OriginFor<T>, mut post: Post<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			post.author = who.clone();

			let hash = post.id.clone();
			// Update storage.
			<Blog<T>>::insert(who.clone(), post);

			let comments_vec: Vec<PostComment<T>> = Vec::new();
			<BlogPostComments<T>>::insert(hash.clone(), comments_vec);

			// Emit an event.
			Self::deposit_event(Event::PostStored(who, hash));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn post_comment(
			origin: OriginFor<T>,
			post_id: T::Hash,
			content: Vec<u8>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<BlogPostComments<T>>::mutate(post_id, |comments| match comments {
				None => Err(()),
				Some(vec) => {
					vec.push(PostComment {
						post_id: post_id.clone(),
						author: who.clone(),
						content,
					});
					Ok(())
				},
			})
			.map_err(|_| <Error<T>>::BlogPostNotFound)?;

			// Emit an event.
			Self::deposit_event(Event::CommentOfPostStored(who, post_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}
