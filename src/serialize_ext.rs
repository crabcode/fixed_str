// fixed_string/src/serialize_ext.rs

//******************************************************************************
//  BinRW Serialization
//******************************************************************************

#[cfg(feature = "binrw")]
mod binrw_ext {
    use crate::*;
    use binrw::io::{Read, Seek, Write};
    use binrw::{BinRead, BinWrite};

    impl<const N: usize> BinRead for FixedStr<N> {
        type Args<'a> = ();

        fn read_options<R: Read + Seek>(
            reader: &mut R,
            _endian: binrw::Endian,
            _args: Self::Args<'_>,
        ) -> binrw::BinResult<Self> {
            let mut buf = [0u8; N];
            reader.read_exact(&mut buf)?;
            Ok(Self { data: buf })
        }
    }

    impl<const N: usize> BinWrite for FixedStr<N> {
        type Args<'a> = ();

        fn write_options<W: Write + Seek>(
            &self,
            writer: &mut W,
            _endian: binrw::Endian,
            _args: Self::Args<'_>,
        ) -> binrw::BinResult<()> {
            writer.write_all(&self.data)?;
            Ok(())
        }
    }
}

// --- Tests ---
#[cfg(all(test, feature = "binrw", feature = "std"))]
mod binrw_tests {
    use crate::*;

    #[test]
    fn test_binrw_roundtrip() {
        use binrw::{BinRead, BinWrite, Endian};
        use std::io::Cursor;

        let original = FixedStr::<5>::new("Hello");
        // Cursor<Vec<u8>> implements both Write and Seek
        let mut cursor = Cursor::new(Vec::new());
        original
            .write_options(&mut cursor, Endian::Little, ())
            .expect("writing failed");
        // Reset the cursor to the beginning for reading
        cursor.set_position(0);
        let read: FixedStr<5> =
            FixedStr::read_options(&mut cursor, Endian::Little, ()).expect("reading failed");
        assert_eq!(original, read);
    }
}

//******************************************************************************
//  Serde Serialization
//******************************************************************************

/// Test module for `serde` integration.
#[cfg(feature = "serde")]
mod serde_ext {
    use crate::*;
    use core::fmt;
    use serde::de::{Error as DeError, Visitor};
    use serde::ser::Error as SerError;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl<const N: usize> Serialize for FixedStr<N> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self.try_as_str() {
                Ok(s) => serializer.serialize_str(&s),
                Err(_) => Err(S::Error::custom(FixedStrError::InvalidUtf8)),
            }
        }
    }

    struct FixedStrVisitor<const N: usize>;

    impl<'de, const N: usize> Visitor<'de> for FixedStrVisitor<N> {
        type Value = FixedStr<N>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a string of at most {} bytes", N)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(FixedStr::new(value))
        }
    }

    impl<'de, const N: usize> Deserialize<'de> for FixedStr<N> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(FixedStrVisitor::<N>)
        }
    }
}

/// Provides byte serialization for `serde`.
#[cfg(feature = "serde")]
pub mod serde_as_bytes {
    use crate::FixedStr;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes `FixedStr<N>` as bytes.
    pub fn serialize<S, const N: usize>(
        value: &FixedStr<N>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(value.as_bytes())
    }

    /// Deserializes `FixedStr<N>` from bytes.
    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<FixedStr<N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: &[u8] = Deserialize::deserialize(deserializer)?;
        FixedStr::<N>::try_from(bytes).map_err(serde::de::Error::custom)
    }
}

// --- Tests ---
/// Test module for `serde` integration.
#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use crate::*;
    use serde::{Deserialize, Serialize};
    use serde_test::{assert_tokens, Token};

    // Define a test struct that uses the as_bytes serialization.
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ByteWrapper {
        #[serde(with = "serialize_ext::serde_as_bytes")]
        inner: FixedStr<5>,
    }

    #[test]
    fn test_serde_as_bytes() {
        let wrapper = ByteWrapper {
            inner: FixedStr::new("Hello"),
        };

        // For a named-field struct, the tokens must include the field name.
        assert_tokens(
            &wrapper,
            &[
                Token::Struct {
                    name: "ByteWrapper",
                    len: 1,
                },
                Token::Str("inner"),
                Token::BorrowedBytes(b"Hello"),
                Token::StructEnd,
            ],
        );
    }
}
