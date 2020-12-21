//! Pure Rust embedded-friendly implementation of the Distinguished Encoding Rules (DER)
//! for Abstract Syntax Notation One (ASN.1) as described in ITU [X.690].
//!
//! # About
//!
//! This crate provides an implementation of a subset of ASN.1 DER necessary
//! for decoding/encoding various cryptography-related formats implemented as
//! part of the [RustCrypto] project, e.g. the [`pkcs8`] crate.
//!
//! The core implementation avoids any heap usage (with convenience methods
//! that allocate gated under the off-by-default `alloc` feature).
//!
//! The implementation is presently specialized for documents which are smaller
//! than 64kB, as that provides a safe bound for the current use cases.
//! However, that may be revisited in the future depending on interest in
//! support for larger documents. Please open a [GitHub Issue] if you find
//! this limit constraining in practice.
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.46** at a minimum.
//!
//! # Usage
//!
//! The following example implements X.509's `AlgorithmIdentifier` message type
//! as defined in [RFC 5280 Section 4.1.1.2].
//!
//! The ASN.1 schema for this message type is as follows:
//!
//! ```text
//!    AlgorithmIdentifier  ::=  SEQUENCE  {
//!         algorithm               OBJECT IDENTIFIER,
//!         parameters              ANY DEFINED BY algorithm OPTIONAL  }
//! ```
//!
//! Structured ASN.1 messages are typically encoded as a `SEQUENCE`, which
//! this crate maps to a Rust struct using the [`Message`] trait. This
//! trait is bounded on the [`Decodable`] trait and provides a blanket impl
//! of the [`Encodable`] trait, so any type which impls [`Message`] can be
//! used for both decoding and encoding.
//!
//! The [`Decoder`] and [`Encoder`] types provide the decoding/encoding API
//! respectively, and are designed to work in conjunction with concrete ASN.1
//! types which impl the [`Decodable`] and [`Encodable`] traits, including
//! all types which impl the [`Message`] trait.
//!
//! The following code example shows how to define a struct which maps to the
//! above schema, as well as impl the [`Message`] trait for that struct:
//!
//! ```
//! # #[cfg(all(feature = "alloc", feature = "oid"))]
//! # {
//! // Note: the following example does not require the `std` feature at all.
//! // It does leverage the `alloc` feature, but also provides instructions for
//! // "heapless" usage when the `alloc` feature is disabled.
//! use core::convert::{TryFrom, TryInto};
//! use der::{Any, Decodable, Encodable, Message, ObjectIdentifier};
//!
//! /// X.509 `AlgorithmIdentifier`.
//! #[derive(Copy, Clone, Debug, Eq, PartialEq)]
//! pub struct AlgorithmIdentifier<'a> {
//!     /// This field contains an ASN.1 `OBJECT IDENTIFIER`, a.k.a. OID.
//!     pub algorithm: ObjectIdentifier,
//!
//!     /// This field is `OPTIONAL` and contains the ASN.1 `ANY` type, which
//!     /// in this example allows arbitrary algorithm-defined parameters.
//!     pub parameters: Option<Any<'a>>
//! }
//!
//! // Note: types which impl `TryFrom<Any<'a>, Error = der::Error>` receive a
//! // blanket impl of the `Decodable` trait, therefore satisfying the
//! // `Decodable` trait bounds on `Message`, which is impl'd below.
//! impl<'a> TryFrom<der::Any<'a>> for AlgorithmIdentifier<'a> {
//!    type Error = der::Error;
//!
//!     fn try_from(any: der::Any<'a>) -> der::Result<AlgorithmIdentifier> {
//!         // The `der::Any::sequence` method asserts that a `der::Any` value
//!         // contains an ASN.1 `SEQUENCE` then calls the provided `FnOnce`
//!         // with a `der::Decoder` which can be used to decode it.
//!         any.sequence(|mut decoder| {
//!             // The `der::Decoder::Decode` method can be used to decode any
//!             // type which impls the `Decodable` trait, which is impl'd for
//!             // all of the ASN.1 built-in types in the `der` crate.
//!             //
//!             // Note that if your struct's fields don't contain an ASN.1
//!             // built-in type specifically, there are also helper methods
//!             // for all of the built-in types supported by this library
//!             // which can be used to select a specific type.
//!             //
//!             // For example, another way of decoding this particular field,
//!             // which contains an ASN.1 `OBJECT IDENTIFIER`, is by calling
//!             // `decoder.oid()`. Similar methods are defined for other
//!             // ASN.1 built-in types.
//!             let algorithm = decoder.decode()?;
//!
//!             // This field contains an ASN.1 `OPTIONAL` type. The `der` crate
//!             // maps this directly to Rust's `Option` type and provides
//!             // impls of the `Decodable` and `Encodable` traits for `Option`.
//!             // To explicitly request an `OPTIONAL` type be decoded, use the
//!             // `decoder.optional()` method.
//!             let parameters = decoder.decode()?;
//!
//!             // The value returned from the provided `FnOnce` will be
//!             // returned from the `any.sequence(...)` call above.
//!             // Note that the entire sequence body *MUST* be consumed
//!             // or an error will be returned.
//!             Ok(Self { algorithm, parameters })
//!         })
//!     }
//! }
//!
//! impl<'a> Message<'a> for AlgorithmIdentifier<'a> {
//!     // The `Message::fields` method is used for encoding and functions as
//!     // a visitor for all of the fields in a message.
//!     //
//!     // To implement it, you must define a slice containing `Encodable`
//!     // trait objects, then pass it to the provided `field_encoder`
//!     // function, which is implemented by the `der` crate and handles
//!     // message serialization.
//!     //
//!     // Trait objects are used because they allow for slices containing
//!     // heterogenous field types, and a callback is used to allow for the
//!     // construction of temporary field encoder types. The latter means
//!     // that the fields of your Rust struct don't necessarily need to
//!     // impl the `Encodable` trait, but if they don't you must construct
//!     // a temporary wrapper value which does.
//!     //
//!     // Types which impl the `Message` trait receive blanket impls of both
//!     // the `Encodable` and `Tagged` traits (where the latter is impl'd as
//!     // `Tagged::TAG = der::Tag::Sequence`.
//!     fn fields<F, T>(&self, field_encoder: F) -> der::Result<T>
//!     where
//!         F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
//!     {
//!         field_encoder(&[&self.algorithm, &self.parameters])
//!     }
//! }
//!
//! // Example parameters value: OID for the NIST P-256 elliptic curve.
//! let parameters = "1.2.840.10045.3.1.7".parse::<ObjectIdentifier>().unwrap();
//!
//! // We need to convert `parameters` into a `der::Any<'a>` type, which wraps
//! // a `&'a [u8]` byte slice.
//! //
//! // To do that, we need owned DER-encoded data so that we can have
//! // `Any` borrow a reference to it, so we have to serialize the OID.
//! //
//! // When the `alloc` feature of this crate is enabled, any type that impls
//! // the `Encodable` trait including all ASN.1 built-in types and any type
//! // which impls `Message` can be serialized by calling `Encodable::to_vec()`.
//! //
//! // If you would prefer to avoid allocations, you can create a byte array
//! // as backing storage instead, pass that to `der::Encoder::new`, and then
//! // encode the `parameters` value using `encoder.encode(parameters)`.
//! let der_encoded_parameters = parameters.to_vec().unwrap();
//!
//! let algorithm_identifier = AlgorithmIdentifier {
//!     // OID for `id-ecPublicKey`, if you're curious
//!     algorithm: "1.2.840.10045.2.1".parse().unwrap(),
//!
//!     // `der::Any<'a>` impls `TryFrom<&'a [u8]>`, which parses the provided
//!     // slice as an ASN.1 DER-encoded message.
//!     parameters: Some(der_encoded_parameters.as_slice().try_into().unwrap())
//! };
//!
//! // Serialize the `AlgorithmIdentifier` created above as ASN.1 DER,
//! // allocating a `Vec<u8>` for storage.
//! //
//! // As mentioned earlier, if you don't have the `alloc` feature enabled you
//! // can create a fix-sized array instead, then call `Encoder::new` with a
//! // refernce to it, then encode the message using
//! // `encoder.encode(algorithm_identifier)`, then finally `encoder.finish()`
//! // to obtain a byte slice containing the encoded message.
//! let der_encoded_algorithm_identifier = algorithm_identifier.to_vec().unwrap();
//!
//! // Deserialize the `AlgorithmIdentifier` we just serialized from ASN.1 DER
//! // using `der::Decodable::from_bytes`.
//! let decoded_algorithm_identifier = AlgorithmIdentifier::from_bytes(
//!     &der_encoded_algorithm_identifier
//! ).unwrap();
//!
//! // Ensure the original `AlgorithmIdentifier` is the same as the one we just
//! // decoded from ASN.1 DER.
//! assert_eq!(algorithm_identifier, decoded_algorithm_identifier);
//! # }
//! ```
//!
//!
//! [X.690]: https://www.itu.int/rec/T-REC-X.690/
//! [RustCrypto]: https://github.com/rustcrypto
//! [`pkcs8`]: https://docs.rs/pkcs8/
//! [GitHub Issue]: https://github.com/RustCrypto/utils/issues
//! [RFC 5280 Section 4.1.1.2]: https://tools.ietf.org/html/rfc5280#section-4.1.1.2

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png",
    html_root_url = "https://docs.rs/der/0.0.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod asn1;
mod byte_slice;
mod decoder;
mod encoder;
mod error;
mod header;
mod length;
mod tag;
mod traits;

pub use crate::{
    asn1::{
        any::Any, bit_string::BitString, integer::RawInteger, null::Null,
        octet_string::OctetString, sequence::Sequence,
    },
    decoder::Decoder,
    encoder::Encoder,
    error::{Error, ErrorKind, Result},
    length::Length,
    tag::Tag,
    traits::{Decodable, Encodable, Message, Tagged},
};

pub(crate) use crate::{byte_slice::ByteSlice, header::Header};

#[cfg(feature = "oid")]
#[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
pub use const_oid::ObjectIdentifier;
