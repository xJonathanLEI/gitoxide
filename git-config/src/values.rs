use std::{borrow::Cow, fmt::Display, str::FromStr};

use serde::{Serialize, Serializer};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Value<'a> {
    Boolean(Boolean),
    Integer(Integer),
    Color(Color),
    Other(Cow<'a, str>),
}

impl<'a> Value<'a> {
    pub fn from_str(s: &'a str) -> Self {
        // if s.
        Self::Other(Cow::Borrowed(s))
    }

    pub fn from_string(s: String) -> Self {
        Self::Other(Cow::Owned(s))
    }
}

impl Serialize for Value<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Boolean(b) => b.serialize(serializer),
            Value::Integer(i) => i.serialize(serializer),
            Value::Color(c) => c.serialize(serializer),
            Value::Other(i) => i.serialize(serializer),
        }
    }
}

// todo display for value

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Boolean {
    True(TrueVariant),
    False(FalseVariant),
}

// todo: Display for boolean

impl Serialize for Boolean {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Boolean::True(_) => serializer.serialize_bool(true),
            Boolean::False(_) => serializer.serialize_bool(false),
        }
    }
}

impl FromStr for Boolean {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = TrueVariant::from_str(value) {
            return Ok(Self::True(v));
        }

        if let Ok(v) = FalseVariant::from_str(value) {
            return Ok(Self::False(v));
        }

        Err(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TrueVariant {
    Yes,
    On,
    True,
    One,
    /// For variables defined without a `= <value>`. This can never be created
    /// from the FromStr trait, as an empty string is false without context.
    Implicit,
}

impl Display for TrueVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yes => write!(f, "yes"),
            Self::On => write!(f, "on"),
            Self::True => write!(f, "true"),
            Self::One => write!(f, "one"),
            Self::Implicit => write!(f, "(implicit)"),
        }
    }
}

impl Serialize for TrueVariant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(true)
    }
}

impl FromStr for TrueVariant {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("yes") {
            Ok(Self::Yes)
        } else if value.eq_ignore_ascii_case("on") {
            Ok(Self::On)
        } else if value.eq_ignore_ascii_case("true") {
            Ok(Self::True)
        } else if value.eq_ignore_ascii_case("one") {
            Ok(Self::One)
        } else {
            Err(())
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FalseVariant {
    No,
    Off,
    False,
    Zero,
    EmptyString,
}

impl Display for FalseVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::No => write!(f, "no"),
            Self::Off => write!(f, "off"),
            Self::False => write!(f, "false"),
            Self::Zero => write!(f, "0"),
            Self::EmptyString => write!(f, "\"\""),
        }
    }
}

impl Serialize for FalseVariant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(false)
    }
}

impl FromStr for FalseVariant {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("no") {
            Ok(Self::No)
        } else if value.eq_ignore_ascii_case("off") {
            Ok(Self::Off)
        } else if value.eq_ignore_ascii_case("false") {
            Ok(Self::False)
        } else if value.eq_ignore_ascii_case("zero") {
            Ok(Self::Zero)
        } else if value.is_empty() {
            Ok(Self::EmptyString)
        } else {
            Err(())
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Integer {
    value: i64,
    suffix: Option<IntegerSuffix>,
}

impl Integer {}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)?;
        if let Some(suffix) = self.suffix {
            write!(f, "{}", suffix)
        } else {
            Ok(())
        }
    }
}

impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(suffix) = self.suffix {
            serializer.serialize_i64(self.value << suffix.bitwise_offset())
        } else {
            serializer.serialize_i64(self.value)
        }
    }
}

// todo from str for integer

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum IntegerSuffix {
    Kilo,
    Mega,
    Giga,
}

impl IntegerSuffix {
    fn bitwise_offset(&self) -> usize {
        match self {
            Self::Kilo => 10,
            Self::Mega => 20,
            Self::Giga => 30,
        }
    }
}

impl Display for IntegerSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kilo => write!(f, "k"),
            Self::Mega => write!(f, "m"),
            Self::Giga => write!(f, "g"),
        }
    }
}

impl Serialize for IntegerSuffix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::Kilo => "k",
            Self::Mega => "m",
            Self::Giga => "g",
        })
    }
}

impl FromStr for IntegerSuffix {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "k" => Ok(Self::Kilo),
            "m" => Ok(Self::Mega),
            "g" => Ok(Self::Giga),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Color {
    foreground: Option<ColorValue>,
    background: Option<ColorValue>,
    attributes: Vec<ColorAttribute>,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(fg) = self.foreground {
            fg.fmt(f)?;
        }

        write!(f, " ")?;

        if let Some(bg) = self.background {
            bg.fmt(f)?;
        }

        self.attributes
            .iter()
            .map(|attr| write!(f, " ").and_then(|_| attr.fmt(f)))
            .collect::<Result<_, _>>()
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum ColorValue {
    Normal,
    Black,
    BrightBlack,
    Red,
    BrightRed,
    Green,
    BrightGreen,
    Yellow,
    BrightYellow,
    Blue,
    BrightBlue,
    Magenta,
    BrightMagenta,
    Cyan,
    BrightCyan,
    White,
    BrightWhite,
    Ansi(u8),
    Rgb(u8, u8, u8),
}

impl Display for ColorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Black => write!(f, "black"),
            Self::BrightBlack => write!(f, "brightblack"),
            Self::Red => write!(f, "red"),
            Self::BrightRed => write!(f, "brightred"),
            Self::Green => write!(f, "green"),
            Self::BrightGreen => write!(f, "brightgreen"),
            Self::Yellow => write!(f, "yellow"),
            Self::BrightYellow => write!(f, "brightyellow"),
            Self::Blue => write!(f, "blue"),
            Self::BrightBlue => write!(f, "brightblue"),
            Self::Magenta => write!(f, "magenta"),
            Self::BrightMagenta => write!(f, "brightmagenta"),
            Self::Cyan => write!(f, "cyan"),
            Self::BrightCyan => write!(f, "brightcyan"),
            Self::White => write!(f, "white"),
            Self::BrightWhite => write!(f, "brightwhite"),
            Self::Ansi(num) => num.fmt(f),
            Self::Rgb(r, g, b) => write!(f, "#{:02x}{:02x}{:02x}", r, g, b),
        }
    }
}

impl Serialize for ColorValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for ColorValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bright = s.starts_with("bright");
        match s {
            "normal" => return Ok(Self::Normal),
            "black" if !bright => return Ok(Self::Black),
            "black" if bright => return Ok(Self::BrightBlack),
            "red" if !bright => return Ok(Self::Red),
            "red" if bright => return Ok(Self::BrightRed),
            "green" if !bright => return Ok(Self::Green),
            "green" if bright => return Ok(Self::BrightGreen),
            "yellow" if !bright => return Ok(Self::Yellow),
            "yellow" if bright => return Ok(Self::BrightYellow),
            "blue" if !bright => return Ok(Self::Blue),
            "blue" if bright => return Ok(Self::BrightBlue),
            "magenta" if !bright => return Ok(Self::Magenta),
            "magenta" if bright => return Ok(Self::BrightMagenta),
            "cyan" if !bright => return Ok(Self::Cyan),
            "cyan" if bright => return Ok(Self::BrightCyan),
            "white" if !bright => return Ok(Self::White),
            "white" if bright => return Ok(Self::BrightWhite),
            _ => (),
        }

        if let Ok(v) = u8::from_str(s) {
            return Ok(Self::Ansi(v));
        }

        if s.starts_with("#") {
            let s = &s[1..];
            if s.len() == 6 {
                let rgb = (
                    u8::from_str_radix(&s[..2], 16),
                    u8::from_str_radix(&s[2..4], 16),
                    u8::from_str_radix(&s[4..], 16),
                );
                match rgb {
                    (Ok(r), Ok(g), Ok(b)) => return Ok(Self::Rgb(r, g, b)),
                    _ => (),
                }
            }
        }

        Err(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ColorAttribute {
    Bold,
    NoBold,
    Dim,
    NoDim,
    Ul,
    NoUl,
    Blink,
    NoBlink,
    Reverse,
    NoReverse,
    Italic,
    NoItalic,
    Strike,
    NoStrike,
}

impl Display for ColorAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bold => write!(f, "bold"),
            Self::NoBold => write!(f, "nobold"),
            Self::Dim => write!(f, "dim"),
            Self::NoDim => write!(f, "nodim"),
            Self::Ul => write!(f, "ul"),
            Self::NoUl => write!(f, "noul"),
            Self::Blink => write!(f, "blink"),
            Self::NoBlink => write!(f, "noblink"),
            Self::Reverse => write!(f, "reverse"),
            Self::NoReverse => write!(f, "noreverse"),
            Self::Italic => write!(f, "italic"),
            Self::NoItalic => write!(f, "noitalic"),
            Self::Strike => write!(f, "strike"),
            Self::NoStrike => write!(f, "nostrike"),
        }
    }
}

impl Serialize for ColorAttribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Bold => "bold",
            Self::NoBold => "nobold",
            Self::Dim => "dim",
            Self::NoDim => "nodim",
            Self::Ul => "ul",
            Self::NoUl => "noul",
            Self::Blink => "blink",
            Self::NoBlink => "noblink",
            Self::Reverse => "reverse",
            Self::NoReverse => "noreverse",
            Self::Italic => "italic",
            Self::NoItalic => "noitalic",
            Self::Strike => "strike",
            Self::NoStrike => "nostrike",
        })
    }
}

impl FromStr for ColorAttribute {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inverted = s.starts_with("no");
        let mut parsed = &s[2..];
        if parsed.starts_with("-") {
            parsed = &parsed[1..];
        }

        match parsed {
            "bold" if !inverted => Ok(Self::Bold),
            "bold" if inverted => Ok(Self::NoBold),
            "dim" if !inverted => Ok(Self::Dim),
            "dim" if inverted => Ok(Self::NoDim),
            "ul" if !inverted => Ok(Self::Ul),
            "ul" if inverted => Ok(Self::NoUl),
            "blink" if !inverted => Ok(Self::Blink),
            "blink" if inverted => Ok(Self::NoBlink),
            "reverse" if !inverted => Ok(Self::Reverse),
            "reverse" if inverted => Ok(Self::NoReverse),
            "italic" if !inverted => Ok(Self::Italic),
            "italic" if inverted => Ok(Self::NoItalic),
            "strike" if !inverted => Ok(Self::Strike),
            "strike" if inverted => Ok(Self::NoStrike),
            _ => Err(()),
        }
    }
}
