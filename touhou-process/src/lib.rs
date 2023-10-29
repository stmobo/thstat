use std::borrow::Borrow;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

use bytemuck::{AnyBitPattern, CheckedBitPattern};

mod data_member;

#[doc(inline)]
pub use data_member::{DataItem, FixedData};

#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

mod private {
    pub trait Sealed {}
}

/// A wrapper around a platform-specific PID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Pid(platform::Pid);

impl Pid {
    pub fn try_into_process_handle(self) -> std::io::Result<ProcessHandle> {
        platform::try_into_process_handle(self.0).map(ProcessHandle)
    }
}

impl From<u32> for Pid {
    fn from(value: u32) -> Self {
        Self(platform::pid_from_u32(value))
    }
}

impl From<Pid> for u32 {
    fn from(value: Pid) -> Self {
        platform::pid_to_u32(value.0)
    }
}

/// A wrapper around a platform-specific process handle.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ProcessHandle(platform::ProcessHandle);

impl TryFrom<Pid> for ProcessHandle {
    type Error = std::io::Error;

    fn try_from(value: Pid) -> Result<Self, Self::Error> {
        value.try_into_process_handle()
    }
}

impl TryFrom<std::process::Child> for ProcessHandle {
    type Error = std::io::Error;

    fn try_from(value: std::process::Child) -> Result<Self, Self::Error> {
        Self::from_child(value)
    }
}

impl ProcessHandle {
    /// Unsafely read memory at `addr` within another process's address space.
    ///
    /// # Safety
    ///
    /// This is effectively a raw pointer read but across processes, and as such
    /// many of the same rules need to be followed. In particular, the caller
    /// must ensure that the read value is valid for type `T`.
    unsafe fn read_unsafe<T: ?Sized>(
        &self,
        addr: NonZeroUsize,
        dest: &mut T,
    ) -> std::io::Result<()> {
        platform::read_unsafe(self.0, addr, dest)
    }

    /// Safely read memory from another process's address space into a mutable reference.
    ///
    /// This is like [`read_unsafe`], but can be called safely because `T` is bound by [`AnyBitPattern`].
    fn read_into<T: AnyBitPattern>(&self, addr: NonZeroUsize, dest: &mut T) -> std::io::Result<()> {
        // SAFETY: The trait bound ensures that any bit pattern is valid for T.
        unsafe { self.read_unsafe(addr, dest) }
    }

    /// Safely read multiple items from another process's address space.
    ///
    /// Much like [`read_into`], this can be called safely thanks to the [`AnyBitPattern`] bound on T.
    fn read_into_slice<T: AnyBitPattern>(
        &self,
        addr: NonZeroUsize,
        dest: &mut [T],
    ) -> std::io::Result<()> {
        // SAFETY: The trait bound ensures that any bit pattern is valid for T.
        unsafe { self.read_unsafe(addr, dest) }
    }

    /// Safely read a value from another process's address space.
    ///
    /// This is a convenience method around [`read_into`] that reads into a temporary value,
    /// then returns the newly read value.
    fn read<T: AnyBitPattern>(&self, addr: NonZeroUsize) -> std::io::Result<T> {
        let mut ret = T::zeroed();
        self.read_into(addr, &mut ret)?;
        Ok(ret)
    }

    /// Safely read memory from another process's address space by checking for valid bit patterns.
    ///
    /// This method safely reads a value of type `T` by first reading raw [`Bits`](CheckedBitPattern::Bits),
    /// checking to see if the result is valid using [`is_valid_bit_pattern`](CheckedBitPattern::is_valid_bit_pattern),
    /// then casting to `T` if true.
    fn read_checked<T: CheckedBitPattern>(&self, addr: NonZeroUsize) -> std::io::Result<Option<T>> {
        let bits = self.read(addr)?;
        if T::is_valid_bit_pattern(&bits) {
            // SAFETY: We checked to ensure a valid bit pattern for the transmute.
            Ok(Some(unsafe {
                std::mem::transmute_copy(&std::mem::ManuallyDrop::new(bits))
            }))
        } else {
            Ok(None)
        }
    }

    /// Get an actual memory location by following a list of offsets.
    fn get_offset<A: Architecture>(
        &self,
        arch: &A,
        offsets: impl IntoIterator<Item = impl Borrow<usize>>,
    ) -> std::io::Result<Option<NonZeroUsize>> {
        let mut offsets = offsets.into_iter().map(|x| *x.borrow());
        let mut address = match offsets.next().map(NonZeroUsize::new) {
            Some(Some(address)) => address,
            Some(None) => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "attempted to get offset from null pointer",
                ));
            }
            None => return Ok(None),
        };

        for offset in offsets {
            address = arch
                .read_pointer(self, address)?
                .ok_or_else(|| {
                    std::io::Error::new(ErrorKind::InvalidData, "encountered null pointer")
                })?
                .checked_add(offset)
                .ok_or_else(|| {
                    std::io::Error::new(ErrorKind::InvalidData, "address calculation overflowed")
                })?;
        }

        Ok(Some(address))
    }

    pub fn new_fixed_item<T: ?Sized, A: Architecture + Default>(
        &self,
        offsets: &'static [usize],
    ) -> FixedData<T, A> {
        FixedData::new(*self, offsets)
    }

    pub fn new_fixed_item_arch<T: ?Sized, A: Architecture>(
        &self,
        arch: A,
        offsets: &'static [usize],
    ) -> FixedData<T, A> {
        FixedData::new_with_arch(*self, arch, offsets)
    }

    pub fn new_data_item<T: ?Sized, A: Architecture + Default>(&self) -> DataItem<T, A> {
        DataItem::new(*self)
    }

    pub fn new_data_item_arch<T: ?Sized, A: Architecture>(&self, arch: A) -> DataItem<T, A> {
        DataItem::new_with_arch(*self, arch)
    }

    pub fn new_data_item_offsets<T: ?Sized, A: Architecture + Default>(
        &self,
        offsets: Vec<usize>,
    ) -> DataItem<T, A> {
        DataItem::new_offsets(*self, offsets)
    }

    pub fn new_data_item_offsets_arch<T: ?Sized, A: Architecture>(
        &self,
        arch: A,
        offsets: Vec<usize>,
    ) -> DataItem<T, A> {
        DataItem::new_offsets_with_arch(*self, arch, offsets)
    }
}

macro_rules! impl_architectures {
    ($($size:literal : $temp_type:ty),*) => {
        impl LittleEndian<1> {
            pub const fn new() -> Self {
                Self(PhantomData)
            }
        }

        impl Default for LittleEndian<1> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        impl Architecture for LittleEndian<1> {
            #[inline(always)]
            fn read_pointer(&self, handle: &ProcessHandle, addr: NonZeroUsize) -> std::io::Result<Option<NonZeroUsize>> {
                handle.read::<u8>(addr).map(|x| x as usize).map(NonZeroUsize::new)
            }
        }

        impl Default for BigEndian<1> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        impl BigEndian<1> {
            pub const fn new() -> Self {
                Self(PhantomData)
            }
        }

        impl Architecture for BigEndian<1> {
            #[inline(always)]
            fn read_pointer(&self, handle: &ProcessHandle, addr: NonZeroUsize) -> std::io::Result<Option<NonZeroUsize>> {
                handle.read::<u8>(addr).map(|x| x as usize).map(NonZeroUsize::new)
            }
        }

        impl NativeEndian<1> {
            pub const fn new() -> Self {
                Self(PhantomData)
            }
        }

        impl Default for NativeEndian<1> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        impl Architecture for NativeEndian<1> {
            #[inline(always)]
            fn read_pointer(&self, handle: &ProcessHandle, addr: NonZeroUsize) -> std::io::Result<Option<NonZeroUsize>> {
                handle.read::<u8>(addr).map(|x| x as usize).map(NonZeroUsize::new)
            }
        }

        $(
            impl LittleEndian<$size> {
                pub const fn new() -> Self {
                    Self(PhantomData)
                }
            }

            impl Default for LittleEndian<$size> {
                fn default() -> Self {
                    Self(PhantomData)
                }
            }

            impl Architecture for LittleEndian<$size> {
                #[inline(always)]
                fn read_pointer(&self, handle: &ProcessHandle, addr: NonZeroUsize) -> std::io::Result<Option<NonZeroUsize>> {
                    handle.read::<[u8; $size]>(addr).map(<$temp_type>::from_le_bytes).map(|x| x as usize).map(NonZeroUsize::new)
                }
            }

            impl BigEndian<$size> {
                pub const fn new() -> Self {
                    Self(PhantomData)
                }
            }

            impl Default for BigEndian<$size> {
                fn default() -> Self {
                    Self(PhantomData)
                }
            }

            impl Architecture for BigEndian<$size> {
                #[inline(always)]
                fn read_pointer(&self, handle: &ProcessHandle, addr: NonZeroUsize) -> std::io::Result<Option<NonZeroUsize>> {
                    handle.read::<[u8; $size]>(addr).map(<$temp_type>::from_be_bytes).map(|x| x as usize).map(NonZeroUsize::new)
                }
            }

            impl NativeEndian<$size> {
                pub const fn new() -> Self {
                    Self(PhantomData)
                }
            }

            impl Default for NativeEndian<$size> {
                fn default() -> Self {
                    Self(PhantomData)
                }
            }

            impl Architecture for NativeEndian<$size> {
                #[inline(always)]
                fn read_pointer(&self, handle: &ProcessHandle, addr: NonZeroUsize) -> std::io::Result<Option<NonZeroUsize>> {
                    handle.read::<[u8; $size]>(addr).map(<$temp_type>::from_ne_bytes).map(|x| x as usize).map(NonZeroUsize::new)
                }
            }
        )*
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct LittleEndian<const WIDTH: usize>(PhantomData<[u8; WIDTH]>);

impl<const WIDTH: usize> private::Sealed for LittleEndian<WIDTH> {}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BigEndian<const WIDTH: usize>(PhantomData<[u8; WIDTH]>);

impl<const WIDTH: usize> private::Sealed for BigEndian<WIDTH> {}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NativeEndian<const WIDTH: usize>(PhantomData<[u8; WIDTH]>);

impl<const WIDTH: usize> private::Sealed for NativeEndian<WIDTH> {}

pub trait Architecture: private::Sealed {
    fn read_pointer(
        &self,
        handle: &ProcessHandle,
        addr: NonZeroUsize,
    ) -> std::io::Result<Option<NonZeroUsize>>;
}

impl_architectures! {
    2: u16,
    4: u32,
    8: u64
}
