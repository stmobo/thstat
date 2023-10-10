//! Types and traits for reading data out of the memory of running Touhou game processes.
//!
//! Aside from extracting basic game state such as player lives and bomb counts,
//! this module also provides support for working with player locations (for example, determining which section of a stage they're playing at the moment).

use std::io;

use sysinfo::{Pid, PidExt, Process, ProcessExt, ProcessRefreshKind, System, SystemExt};

use crate::types::Game;

#[doc(hidden)]
pub mod traits;
#[doc(hidden)]
pub mod types;

#[doc(inline)]
pub use traits::*;
#[doc(inline)]
pub use types::*;

pub type Result<G, T> = std::result::Result<T, MemoryReadError<G>>;

pub(crate) fn try_into_or_mem_error<G, T, U>(val: T) -> Result<G, U>
where
    G: Game,
    T: TryInto<U>,
    T::Error: Into<types::MemoryReadError<G>>,
{
    val.try_into().map_err(T::Error::into)
}

macro_rules! ensure_float_within_range {
    ($x:expr => $t:ty : ($lo:literal, $hi:literal, $val_name:literal)) => {{
        use crate::memory::MemoryReadError;

        let x = ($x).trunc();
        if x < ($lo as f32) || x > ($hi as f32) {
            return Err(MemoryReadError::float_out_of_range(
                $val_name, x, $lo as f32, $hi as f32,
            ));
        } else {
            x as $t
        }
    }};
}

macro_rules! define_state_struct {
    {
        $struct_name:ident {
            $($field_name:ident: $field_type:ty),*$(,)?
        }
    } => {
        #[derive(Debug, Clone, Copy)]
        pub struct $struct_name {
            $($field_name: $field_type),*
        }

        #[automatically_derived]
        impl $struct_name {
            $(
                pub fn $field_name(&self) -> $field_type {
                    self.$field_name
                }
            )*
        }
    };
}

pub trait ProcessAttached: Sized {
    fn from_pid(pid: u32) -> io::Result<Self>;
    fn is_attachable_process(proc: &Process) -> bool;
}

#[derive(Debug)]
pub struct Attached<T> {
    system: System,
    pid: Pid,
    inner: T,
}

impl<T: ProcessAttached> Attached<T> {
    fn find_process(system: &System) -> Option<&Process> {
        system
            .processes()
            .iter()
            .map(|(_, process)| process)
            .find(|proc| T::is_attachable_process(proc))
            .filter(|proc| proc.run_time() > 15)
    }

    pub fn new() -> io::Result<Option<Self>> {
        let mut system = System::new();
        system.refresh_processes_specifics(ProcessRefreshKind::new());

        if let Some(proc) = Self::find_process(&system) {
            let inner = T::from_pid(proc.pid().as_u32())?;
            Ok(Some(Self {
                pid: proc.pid(),
                system,
                inner,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid.as_u32()
    }

    pub fn is_running(&mut self) -> bool {
        self.system
            .refresh_process_specifics(self.pid, ProcessRefreshKind::new())
    }

    pub fn access(&mut self) -> Option<&T> {
        if self.is_running() {
            Some(&self.inner)
        } else {
            None
        }
    }
}

pub(crate) use {define_state_struct, ensure_float_within_range};
