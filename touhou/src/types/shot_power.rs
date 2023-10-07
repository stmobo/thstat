//! Types for working with player shot power values.

use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::errors::InvalidShotPower;
use super::Game;

/// A trait for types that represent power values in Touhou games.
///
/// This trait is similar to the [`super::GameValue`] trait used for other game-specific types.
pub trait PowerValue:
    Debug + Display + Ord + Hash + Serialize + DeserializeOwned + Copy + Sync + Send + Unpin + 'static
{
    /// The value representing maximum shot power.
    const MAX_POWER: Self;

    /// The underlying representation for this shot power.
    ///
    /// For the first-generation Windows games, this is `u8`, and for the second-generation games, this is `u16`.
    type RawValue;

    /// Create a new shot power instance from a raw value.
    fn new(value: Self::RawValue) -> Result<Self, InvalidShotPower>;

    /// Get the raw value associated with this shot power.
    fn unwrap(self) -> Self::RawValue;

    // Get whether this shot power represents maximum power.
    fn is_max(self) -> bool {
        self == Self::MAX_POWER
    }
}

/// Represents a shot power value from the first generation Windows games (i.e. games 6, 7, and 8).
///
/// Shot powers in these games are integers from 0 to 128 inclusive.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    std::hash::Hash,
    Default,
    Serialize,
    Deserialize,
)]
#[serde(try_from = "u8", into = "u8")]
#[repr(transparent)]
pub struct Gen1Power(u8);

impl Gen1Power {
    pub const MAX_POWER: Self = Self(128);

    pub const fn new(value: u8) -> Result<Self, InvalidShotPower> {
        if value <= 128 {
            Ok(Self(value))
        } else {
            Err(InvalidShotPower::InvalidPower(value as u16, 128))
        }
    }

    pub const fn unwrap(self) -> u8 {
        self.0
    }

    pub const fn is_max(self) -> bool {
        self.0 == 128
    }
}

impl TryFrom<u8> for Gen1Power {
    type Error = InvalidShotPower;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<u16> for Gen1Power {
    type Error = InvalidShotPower;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if let Ok(conv) = value.try_into() {
            Self::new(conv)
        } else {
            Err(InvalidShotPower::InvalidPower(value, 128))
        }
    }
}

impl From<Gen1Power> for u8 {
    fn from(value: Gen1Power) -> Self {
        value.0
    }
}

impl From<Gen1Power> for u16 {
    fn from(value: Gen1Power) -> Self {
        value.0 as u16
    }
}

impl AsRef<u8> for Gen1Power {
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

impl Deref for Gen1Power {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<u8> for Gen1Power {
    fn borrow(&self) -> &u8 {
        &self.0
    }
}

impl PartialEq<u8> for Gen1Power {
    fn eq(&self, other: &u8) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u8> for Gen1Power {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(other))
    }
}

impl Display for Gen1Power {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_max() {
            f.write_str("MAX")
        } else {
            write!(f, "{} / 128", self.0)
        }
    }
}

impl PowerValue for Gen1Power {
    const MAX_POWER: Self = Self(128);

    type RawValue = u8;

    fn new(value: u8) -> Result<Self, InvalidShotPower> {
        if value <= 128 {
            Ok(Self(value))
        } else {
            Err(InvalidShotPower::InvalidPower(value as u16, 128))
        }
    }

    fn unwrap(self) -> u8 {
        self.0
    }
}

/// Represents a shot power value from most of the second generation Windows games (specifically 12 and onwards).
///
/// Shot powers in these games are decimal values that increase and decrease in increments of 0.01.
///
/// Note that Touhou 10 has its own [`ShotPower`](crate::th10::ShotPower) type.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(try_from = "u16", into = "u16")]
#[repr(transparent)]
pub struct Gen2Power<const MAX: u16>(u16);

impl<const MAX: u16> Gen2Power<MAX> {
    pub const MAX_POWER: Self = Self(MAX);

    pub const fn new(value: u16) -> Result<Self, InvalidShotPower> {
        if value <= MAX {
            Ok(Self(value))
        } else {
            Err(InvalidShotPower::InvalidPower(value, MAX))
        }
    }

    pub const fn unwrap(self) -> u16 {
        self.0
    }

    pub const fn is_max(self) -> bool {
        self.0 == MAX
    }
}

impl<const MAX: u16> TryFrom<u16> for Gen2Power<MAX> {
    type Error = InvalidShotPower;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const MAX: u16> From<Gen2Power<MAX>> for u16 {
    fn from(value: Gen2Power<MAX>) -> Self {
        value.0
    }
}

impl<const MAX: u16> From<Gen2Power<MAX>> for f64 {
    fn from(value: Gen2Power<MAX>) -> Self {
        (value.0 as Self) * 0.01
    }
}

impl<const MAX: u16> From<Gen2Power<MAX>> for f32 {
    fn from(value: Gen2Power<MAX>) -> Self {
        (value.0 as Self) * 0.01
    }
}

impl<const MAX: u16> AsRef<u16> for Gen2Power<MAX> {
    fn as_ref(&self) -> &u16 {
        &self.0
    }
}

impl<const MAX: u16> Deref for Gen2Power<MAX> {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const MAX: u16> Borrow<u16> for Gen2Power<MAX> {
    fn borrow(&self) -> &u16 {
        &self.0
    }
}

impl<const MAX: u16> PartialEq<u16> for Gen2Power<MAX> {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}

impl<const MAX: u16> PartialOrd<u16> for Gen2Power<MAX> {
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(other))
    }
}

fn fmt_decimal_power(f: &mut std::fmt::Formatter<'_>, raw_value: u16) -> std::fmt::Result {
    let whole = raw_value / 100;
    let frac = raw_value % 100;
    write!(f, "{}.{:02}", whole, frac)
}

impl<const MAX: u16> Display for Gen2Power<MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_decimal_power(f, self.0)?;
        f.write_str(" / ")?;
        fmt_decimal_power(f, MAX)
    }
}

impl<const MAX: u16> PowerValue for Gen2Power<MAX> {
    const MAX_POWER: Self = Self(MAX);

    type RawValue = u16;

    fn new(value: u16) -> Result<Self, InvalidShotPower> {
        if value <= MAX {
            Ok(Self(value))
        } else {
            Err(InvalidShotPower::InvalidPower(value, MAX))
        }
    }

    fn unwrap(self) -> u16 {
        self.0
    }
}
/// Represents the in-game power of a shot from one of the Touhou games.
///
/// This is a convenience wrapper around the game-specific shot power types in this crate,
/// such as [`Gen1Power`], [`Gen2Power`], and Touhou 10's specific [`ShotPower`](crate::th10::ShotPower) type.
///
/// To access the inner type, use [`Self::unwrap`].

#[repr(transparent)]
pub struct ShotPower<G: Game>(G::ShotPower);

impl<G: Game> ShotPower<G> {
    /// Wraps a game-specific shot power value in a new wrapper instance.
    pub const fn new(value: G::ShotPower) -> Self {
        Self(value)
    }

    /// Gets the inner game-specific power type contained in this wrapper.
    pub const fn unwrap(self) -> G::ShotPower {
        self.0
    }

    /// Gets the wrapped value directly as a primitive.
    ///
    /// For the 1st-generation windows games (EoSD, PCB, and IN) this will be a `u8`,
    /// and for later games this will be a `u16`.
    pub fn raw_value(self) -> <G::ShotPower as PowerValue>::RawValue {
        self.0.unwrap()
    }
}

impl<G: Game> Clone for ShotPower<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: Game> Copy for ShotPower<G> {}

impl<G: Game> Ord for ShotPower<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<G: Game> PartialOrd for ShotPower<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<G: Game> PartialEq for ShotPower<G> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<G: Game> Eq for ShotPower<G> {}

impl<G: Game> Hash for ShotPower<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<G: Game> Serialize for ShotPower<G> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, G: Game> Deserialize<'de> for ShotPower<G> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <G::ShotPower as Deserialize<'de>>::deserialize(deserializer).map(Self)
    }
}

impl<G: Game> AsRef<G::ShotPower> for ShotPower<G> {
    fn as_ref(&self) -> &G::ShotPower {
        &self.0
    }
}

impl<G: Game> Deref for ShotPower<G> {
    type Target = G::ShotPower;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<G: Game<ShotPower = Gen1Power>> TryFrom<u8> for ShotPower<G> {
    type Error = InvalidShotPower;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Gen1Power::try_from(value).map(Self)
    }
}

impl<G: Game<ShotPower = Gen1Power>> TryFrom<u16> for ShotPower<G> {
    type Error = InvalidShotPower;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Gen1Power::try_from(value).map(Self)
    }
}

impl<G: Game<ShotPower = Gen1Power>> From<ShotPower<G>> for u8 {
    fn from(value: ShotPower<G>) -> Self {
        value.unwrap().into()
    }
}

impl<G: Game<ShotPower = Gen1Power>> From<ShotPower<G>> for u16 {
    fn from(value: ShotPower<G>) -> Self {
        value.unwrap().into()
    }
}

impl<G: Game<ShotPower = Gen1Power>> Borrow<u8> for ShotPower<G> {
    fn borrow(&self) -> &u8 {
        self.0.as_ref()
    }
}

impl<G: Game<ShotPower = Gen1Power>> PartialEq<u8> for ShotPower<G> {
    fn eq(&self, other: &u8) -> bool {
        self.0.eq(other)
    }
}

impl<G: Game<ShotPower = Gen1Power>> PartialOrd<u8> for ShotPower<G> {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<const MAX: u16, G: Game<ShotPower = Gen2Power<MAX>>> From<ShotPower<G>> for f32 {
    fn from(value: ShotPower<G>) -> Self {
        value.unwrap().into()
    }
}

impl<const MAX: u16, G: Game<ShotPower = Gen2Power<MAX>>> From<ShotPower<G>> for f64 {
    fn from(value: ShotPower<G>) -> Self {
        value.unwrap().into()
    }
}

impl<const MAX: u16, G: Game<ShotPower = Gen2Power<MAX>>> Borrow<u16> for ShotPower<G> {
    fn borrow(&self) -> &u16 {
        self.0.as_ref()
    }
}

impl<const MAX: u16, G: Game<ShotPower = Gen2Power<MAX>>> PartialEq<u16> for ShotPower<G> {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}

impl<const MAX: u16, G: Game<ShotPower = Gen2Power<MAX>>> PartialOrd<u16> for ShotPower<G> {
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
