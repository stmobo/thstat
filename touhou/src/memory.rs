use std::error::Error;
use std::io::{self, Error as IOError, ErrorKind, Result as IOResult};

use sysinfo::{Pid, PidExt, Process, ProcessExt, ProcessRefreshKind, System, SystemExt};

pub mod traits;
pub mod types;

pub use traits::*;
pub use types::*;

pub(crate) fn try_into_or_io_error<T, U>(kind: ErrorKind) -> impl FnOnce(T) -> IOResult<U>
where
    T: TryInto<U>,
    T::Error: Into<Box<dyn Error + Send + Sync>>,
{
    move |val| {
        val.try_into()
            .map_err(move |error| IOError::new(kind, error))
    }
}

macro_rules! ensure_float_within_range {
    ($x:expr => $t:ty : ($lo:literal, $hi:literal, $val_name:literal)) => {{
        let x = ($x).trunc();
        if x < ($lo as f32) || x > ($hi as f32) {
            return Err(IOError::new(
                ErrorKind::InvalidData,
                format!(
                    "{} not in expected range (got {}, expected {}..={})",
                    $val_name, x, $lo, $hi
                ),
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
    fn from_pid(pid: u32) -> IOResult<Self>;
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

    pub fn new() -> IOResult<Option<Self>> {
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

    pub fn access(&mut self) -> IOResult<&T> {
        if self.is_running() {
            Ok(&self.inner)
        } else {
            Err(io::Error::new(
                ErrorKind::ConnectionAborted,
                format!("process {} is no longer running", self.pid.as_u32()),
            ))
        }
    }
}

pub(crate) use {define_state_struct, ensure_float_within_range};
