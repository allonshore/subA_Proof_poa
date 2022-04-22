#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	// use frame_support::traits::Vec;
	// use frame_support::traits::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}



	//定义存储单元，proofs 用来存储存证 key是u32 是一个hash值 owner，value是2个元素组成的tuple，第一个是用户id，第二个是区块
	// #[pallet::storage]
	// #[pallet::getter(fn proofs)]
	// pub(super) type Proofs<T: Config> =
	// 	StorageMap<_, Blake2_128Concat, Vec<u8> ,(T::AccountId, T::BlockNumber)>;
	#[pallet::storage]
	pub(super) type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		(T::AccountId, T::BlockNumber),
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimedTransfered(T::AccountId,T::AccountId,Vec<u8>),
    }

	#[pallet::error]
	pub enum Error<T> {
        ProofAlreadyExist,
        ClaimNotExist,
		NotClaimOwner,
		NoSuchProof,
		NotProofOwner,
    }

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//创建存证可调用函数  origin表示发送方 claim存证的hash值
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			//校验发送方是一个签名的
			let sender = ensure_signed(origin)?;
			//判断是否存在一个这样的存证
			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);
			//存储
			Proofs::<T>::insert(
				&claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number())
			);
			//
			Self::deposit_event(Event::ClaimCreated(sender, claim));
			Ok(().into())
		}
        //销毁存证
        #[pallet::weight(0)]
        pub fn revoke_claim(origin: OriginFor<T>,claim:Vec<u8>) -> DispatchResultWithPostInfo{
            let sender = ensure_signed(origin)?;

            let(owner,_) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
            ensure!(owner == sender, Error::<T>::NotClaimOwner);
            //删除记录
            Proofs::<T>::remove(&claim);
            Self::deposit_event(Event::ClaimRevoked(sender,claim));
            Ok(().into())
        }

		#[pallet::weight(0)]
		pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// 检测存证文件是否存在
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::NoSuchProof);

			let (owner, _) = Proofs::<T>::get(&claim).expect("All proofs must have an owner!");

			ensure!(owner == sender, Error::<T>::NotProofOwner);

			Proofs::<T>::insert(&claim, (dest.clone(), frame_system::Pallet::<T>::block_number()));
			// 发送事件，声明权证转移
			Self::deposit_event(Event::ClaimedTransfered(sender,dest,claim));

			Ok(().into())
		}

	}
}
