use std::marker::PhantomData;
use std::num::NonZeroUsize;

use bytemuck::{AnyBitPattern, CheckedBitPattern};

use crate::{Architecture, ProcessHandle};

#[derive(Debug, Copy, Clone)]
pub struct FixedData<T: ?Sized + 'static, A: Architecture> {
    offsets: &'static [usize],
    handle: ProcessHandle,
    arch: A,
    _marker: PhantomData<&'static T>,
}

impl<T: ?Sized + 'static, A: Architecture> FixedData<T, A> {
    pub const fn new_with_arch(handle: ProcessHandle, arch: A, offsets: &'static [usize]) -> Self {
        Self {
            offsets,
            handle,
            arch,
            _marker: PhantomData,
        }
    }

    pub fn get_address(&self) -> std::io::Result<NonZeroUsize> {
        self.handle
            .get_offset(&self.arch, self.offsets)
            .map(Option::unwrap)
    }

    /// Unsafely read this item into a mutable reference.
    ///
    /// # Safety
    ///
    /// This is effectively a raw pointer read but across processes, and as such
    /// many of the same rules need to be followed. In particular, the caller
    /// must ensure that the read value is valid for type `T`.
    pub unsafe fn read_unsafe(&self, dest: &mut T) -> std::io::Result<()> {
        self.handle.read_unsafe(self.get_address()?, dest)
    }
}

impl<T: ?Sized + 'static, A: Architecture + Default> FixedData<T, A> {
    pub fn new(handle: ProcessHandle, offsets: &'static [usize]) -> Self {
        Self::new_with_arch(handle, Default::default(), offsets)
    }
}

impl<T: CheckedBitPattern, A: Architecture> FixedData<T, A> {
    pub fn read_checked(&self) -> std::io::Result<Option<T>> {
        self.get_address()
            .and_then(|addr| self.handle.read_checked(addr))
    }
}

impl<T: AnyBitPattern, A: Architecture> FixedData<T, A> {
    pub fn read_into(&self, dest: &mut T) -> std::io::Result<()> {
        self.get_address()
            .and_then(|addr| self.handle.read_into(addr, dest))
    }

    pub fn read_into_slice(&self, dest: &mut [T]) -> std::io::Result<()> {
        self.get_address()
            .and_then(|addr| self.handle.read_into_slice(addr, dest))
    }

    pub fn read(&self) -> std::io::Result<T> {
        self.get_address().and_then(|addr| self.handle.read(addr))
    }
}

#[derive(Debug, Clone)]
pub struct DataItem<T: ?Sized + 'static, A: Architecture> {
    offsets: Vec<usize>,
    arch: A,
    handle: ProcessHandle,
    _marker: PhantomData<(&'static T, A)>,
}

impl<T: ?Sized + 'static, A: Architecture> DataItem<T, A> {
    pub const fn new_with_arch(handle: ProcessHandle, arch: A) -> Self {
        Self {
            handle,
            arch,
            offsets: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub const fn new_offsets_with_arch(
        handle: ProcessHandle,
        arch: A,
        offsets: Vec<usize>,
    ) -> Self {
        Self {
            handle,
            arch,
            offsets,
            _marker: PhantomData,
        }
    }

    pub fn get_address(&self) -> std::io::Result<Option<NonZeroUsize>> {
        self.handle.get_offset(&self.arch, &self.offsets)
    }

    /// Unsafely read this item into a mutable reference.
    ///
    /// # Safety
    ///
    /// This is effectively a raw pointer read but across processes, and as such
    /// many of the same rules need to be followed. In particular, the caller
    /// must ensure that the read value is valid for type `T`.
    pub unsafe fn read_unsafe(&self, dest: &mut T) -> std::io::Result<()> {
        if let Some(addr) = self.get_address()? {
            self.handle.read_unsafe(addr, dest)
        } else {
            Ok(())
        }
    }
}

impl<T: ?Sized + 'static, A: Architecture + Default> DataItem<T, A> {
    pub fn new(handle: ProcessHandle) -> Self {
        Self::new_with_arch(handle, Default::default())
    }

    pub fn new_offsets(handle: ProcessHandle, offsets: Vec<usize>) -> Self {
        Self::new_offsets_with_arch(handle, Default::default(), offsets)
    }
}

impl<T: CheckedBitPattern, A: Architecture> DataItem<T, A> {
    pub fn read_checked(&self) -> std::io::Result<Option<T>> {
        self.get_address()?
            .and_then(|addr| self.handle.read_checked(addr).transpose())
            .transpose()
    }
}

impl<T: AnyBitPattern, A: Architecture> DataItem<T, A> {
    pub fn read_into(&self, dest: &mut T) -> std::io::Result<()> {
        if let Some(addr) = self.get_address()? {
            self.handle.read_into(addr, dest)
        } else {
            Ok(())
        }
    }

    pub fn read_into_slice(&self, dest: &mut [T]) -> std::io::Result<()> {
        if let Some(addr) = self.get_address()? {
            self.handle.read_into_slice(addr, dest)
        } else {
            Ok(())
        }
    }

    pub fn read(&self) -> std::io::Result<Option<T>> {
        self.get_address()?
            .map(|addr| self.handle.read(addr))
            .transpose()
    }
}
