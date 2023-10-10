use std::error::Error;
use std::fmt;
use std::num::TryFromIntError;

use super::GameId;

#[derive(Debug, Copy, Clone)]
pub struct OutOfRangeError<T> {
    value: T,
    valid_start: T,
    valid_end: T,
}

impl<T> OutOfRangeError<T> {
    pub(crate) fn from_other<U: Into<T>>(src: OutOfRangeError<U>) -> Self {
        Self {
            value: src.value.into(),
            valid_start: src.valid_start.into(),
            valid_end: src.valid_end.into(),
        }
    }
}

impl<T: fmt::Display + fmt::Debug> OutOfRangeError<T> {
    pub const fn new(value: T, valid_start: T, valid_end: T) -> Self {
        Self {
            value,
            valid_start,
            valid_end,
        }
    }
}

impl<T: fmt::Display + fmt::Debug> fmt::Display for OutOfRangeError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = &self.value;
        let start = &self.valid_start;
        let end = &self.valid_end;
        write!(
            f,
            "{value} is out of range (valid values are {start}..={end})"
        )
    }
}

impl<T: fmt::Display + fmt::Debug> Error for OutOfRangeError<T> {}

#[derive(Debug, Copy, Clone)]
pub struct InvalidGameId(u8);

impl InvalidGameId {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for InvalidGameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid game ID {}", self.0)
    }
}

impl Error for InvalidGameId {}

#[derive(Debug, Copy, Clone)]
pub enum ValueErrorReason<T> {
    OutOfRange(OutOfRangeError<T>),
    IntConversion(TryFromIntError),
    WrongGame(GameId),
    GameNotSupported,
}

impl<T> ValueErrorReason<T> {
    pub(crate) fn from_other<U: Into<T>>(src: ValueErrorReason<U>) -> Self {
        match src {
            ValueErrorReason::OutOfRange(err) => Self::OutOfRange(OutOfRangeError::from_other(err)),
            ValueErrorReason::IntConversion(err) => Self::IntConversion(err),
            ValueErrorReason::WrongGame(id) => Self::WrongGame(id),
            ValueErrorReason::GameNotSupported => Self::GameNotSupported,
        }
    }
}

impl<T: fmt::Display + fmt::Debug + 'static> ValueErrorReason<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OutOfRange(err) => Some(err),
            Self::IntConversion(err) => Some(err),
            Self::WrongGame(_) | Self::GameNotSupported => None,
        }
    }
}

impl<T: fmt::Display + fmt::Debug + 'static> fmt::Display for ValueErrorReason<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfRange(err) => err.fmt(f),
            Self::IntConversion(err) => err.fmt(f),
            Self::WrongGame(err) => write!(f, "value is for {}", err.abbreviation()),
            Self::GameNotSupported => "support not compiled".fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InvalidGameValue<T: 'static> {
    type_name: &'static str,
    game: GameId,
    reason: ValueErrorReason<T>,
}

impl<T: 'static> InvalidGameValue<T> {
    pub(crate) fn from_other<U: Into<T>>(src: InvalidGameValue<U>) -> Self {
        Self {
            type_name: src.type_name,
            game: src.game,
            reason: ValueErrorReason::from_other(src.reason),
        }
    }

    pub(crate) fn into_other<U>(self) -> InvalidGameValue<U>
    where
        U: 'static,
        T: Into<U>,
    {
        InvalidGameValue::from_other(self)
    }
}

impl<T: fmt::Display + fmt::Debug + 'static> InvalidGameValue<T> {
    pub const fn new(type_name: &'static str, game: GameId, reason: ValueErrorReason<T>) -> Self {
        Self {
            type_name,
            game,
            reason,
        }
    }

    pub const fn out_of_range(
        type_name: &'static str,
        game: GameId,
        value: T,
        valid_start: T,
        valid_end: T,
    ) -> Self {
        Self::new(
            type_name,
            game,
            ValueErrorReason::OutOfRange(OutOfRangeError::new(value, valid_start, valid_end)),
        )
    }

    pub const fn int_conversion(
        type_name: &'static str,
        game: GameId,
        err: TryFromIntError,
    ) -> Self {
        Self::new(type_name, game, ValueErrorReason::IntConversion(err))
    }

    pub const fn wrong_game(type_name: &'static str, expected: GameId, actual: GameId) -> Self {
        Self::new(type_name, expected, ValueErrorReason::WrongGame(actual))
    }

    pub const fn game_not_supported(type_name: &'static str, game: GameId) -> Self {
        Self::new(type_name, game, ValueErrorReason::GameNotSupported)
    }
}

impl<T: fmt::Display + fmt::Debug + 'static> From<std::convert::Infallible>
    for InvalidGameValue<T>
{
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

impl<T: fmt::Display + fmt::Debug + 'static> fmt::Display for InvalidGameValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = self.type_name;
        let game = self.game.abbreviation();
        let reason = &self.reason;
        write!(f, "invalid {type_name} value for {game}: {reason}")
    }
}

impl<T: fmt::Display + fmt::Debug + 'static> Error for InvalidGameValue<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.reason.source()
    }
}

macro_rules! define_value_error {
    ($ty_vis:vis $err_ty:ident : $val_ty:ty, $type_name:literal) => {
            #[repr(transparent)]
            $ty_vis struct $err_ty<G: crate::types::Game>(crate::types::errors::InvalidGameValue<$val_ty>, std::marker::PhantomData<G>);

            #[automatically_derived]
            impl<G: crate::types::Game> $err_ty<G> {
                $ty_vis const fn out_of_range(value: $val_ty, valid: std::ops::RangeInclusive<$val_ty>) -> Self {
                    use crate::types::errors::InvalidGameValue;
                    use std::marker::PhantomData;
                    Self(InvalidGameValue::out_of_range($type_name, G::GAME_ID, value, *valid.start(), *valid.end()), PhantomData)
                }

                $ty_vis const fn wrong_game(actual: crate::types::GameId) -> Self {
                    use crate::types::errors::InvalidGameValue;
                    use std::marker::PhantomData;
                    Self(InvalidGameValue::wrong_game($type_name, G::GAME_ID, actual), PhantomData)
                }

                $ty_vis const fn game_not_supported() -> Self {
                    use crate::types::errors::InvalidGameValue;
                    use std::marker::PhantomData;
                    Self(InvalidGameValue::game_not_supported($type_name, G::GAME_ID), PhantomData)
                }

                $ty_vis const fn into_inner(self) -> crate::types::errors::InvalidGameValue<$val_ty> {
                    self.0
                }
            }

            impl<G: crate::types::Game> From<$err_ty<G>> for crate::types::errors::InvalidGameValue<$val_ty> {
                fn from(value: $err_ty<G>) -> Self {
                    value.0
                }
            }

            impl<G: crate::types::Game> Clone for $err_ty<G> {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl<G: crate::types::Game> Copy for $err_ty<G> { }

            impl<G: crate::types::Game> From<std::num::TryFromIntError> for $err_ty<G> {
                fn from(value: std::num::TryFromIntError) -> Self {
                    use crate::types::errors::InvalidGameValue;
                    use std::marker::PhantomData;
                    Self(InvalidGameValue::int_conversion($type_name, G::GAME_ID, value), PhantomData)
                }
            }

            impl<G: crate::types::Game> From<std::convert::Infallible> for $err_ty<G> {
                fn from(value: std::convert::Infallible) -> Self {
                    match value { }
                }
            }

            impl<G: crate::types::Game> AsRef<crate::types::errors::InvalidGameValue<$val_ty>> for $err_ty<G> {
                fn as_ref(&self) -> &crate::types::errors::InvalidGameValue<$val_ty> {
                    &self.0
                }
            }

            impl<G: crate::types::Game> std::borrow::Borrow<crate::types::errors::InvalidGameValue<$val_ty>> for $err_ty<G> {
                fn borrow(&self) -> &crate::types::errors::InvalidGameValue<$val_ty> {
                    &self.0
                }
            }

            impl<G: crate::types::Game> std::ops::Deref for $err_ty<G> {
                type Target = crate::types::errors::InvalidGameValue<$val_ty>;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<G: crate::types::Game> std::fmt::Display for $err_ty<G> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl<G: crate::types::Game> std::fmt::Debug for $err_ty<G> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl<G: crate::types::Game> std::error::Error for $err_ty<G> {
                fn source(&self) -> Option<&(dyn Error + 'static)> {
                    self.0.source()
                }
            }
    }
}

define_value_error!(pub InvalidShotType : u16, "shot type");
define_value_error!(pub InvalidCardId : u32, "spell card");
define_value_error!(pub InvalidStageId : u16, "stage");
define_value_error!(pub InvalidDifficultyId : u16, "difficulty");
define_value_error!(pub InvalidShotPower : u16, "shot power");
