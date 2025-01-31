// LNP/BP client-side-validation foundation libraries implementing LNPBP
// specifications & standards (LNPBP-4, 7, 8, 9, 42, 81)
//
// Written in 2019-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.
//
// The author of the code acknowledges significant input from Peter Todd,
// who is the author of single-use-seal concept and who spent a lot of his time
// to help to understanding single-use-seal concept and write the current
// implementation.

// Coding conventions
#![recursion_limit = "256"]
#![deny(dead_code, missing_docs, warnings)]
#![allow(clippy::needless_borrow)] // Due to amplify_derive::Display bug

//! # Single-use-seals
//!
//! Set of traits that allow to implement Peter's Todd **single-use seal**
//! paradigm. Information in this file partially contains extracts from Peter's
//! works listed in "Further reading" section.
//!
//! ## Single-use-seal definition
//!
//! Analogous to the real-world, physical, single-use-seals used to secure
//! shipping containers, a single-use-seal primitive is a unique object that can
//! be closed over a message exactly once. In short, a single-use-seal is an
//! abstract mechanism to prevent double-spends.
//!
//! A single-use-seal implementation supports two fundamental operations:
//! * `Close(l,m) → w` — Close seal l over message m, producing a witness `w`.
//! * `Verify(l,w,m) → bool` — Verify that the seal l was closed over message
//! `m`.
//!
//! A single-use-seal implementation is secure if it is impossible for an
//! attacker to cause the Verify function to return true for two distinct
//! messages m1, m2, when applied to the same seal (it is acceptable, although
//! non-ideal, for there to exist multiple witnesses for the same seal/message
//! pair).
//!
//! Practical single-use-seal implementations will also obviously require some
//! way of generating new single-use-seals:
//! * `Gen(p)→l` — Generate a new seal basing on some seal definition data `p`.
//!
//! ## Terminology
//!
//! **Single-use-seal**: a commitment to commit to some (potentially unknown)
//!   message. The first commitment (i.e. single-use-seal) must be a
//!   well-defined (i.e. fully specified and unequally identifiable
//!   in some space, like in time/place or within a given formal informational
//!   system).
//! **Closing of a single-use-seal over message**: a fulfilment of the first
//!   commitment: creation of the actual commitment to some message in a form
//!   unequally defined by the seal.
//! **Witness**: data produced with closing of a single use seal which are
//!   required and sufficient for an independent party to verify that the seal
//!   was indeed closed over a given message (i.e. the commitment to the message
//!   had being created according to the seal definition).
//!
//! NB: It's important to note, that while its possible to deterministically
//!   define was a given seal closed it yet may be not possible to find out
//!   if the seal is open; i.e. seal status may be either "closed over message"
//!   or "unknown". Some specific implementations of single-use-seals may define
//!   procedure to deterministically prove that a given seal is not closed (i.e.
//!   opened), however this is not a part of the specification and we should
//!   not rely on the existence of such possibility in all cases.
//!
//! ## Trait structure
//!
//! The module defines trait [`SingleUseSeal`] that can be used for
//! implementation of single-use-seals with methods for seal close and
//! verification. A type implementing this trait operates only with messages
//! (which is represented by any type that implements `AsRef<[u8]>`,i.e. can be
//! represented as a sequence of bytes) and witnesses (which is represented by
//! an associated type [`SingleUseSeal::Witness`]). At the same time,
//! [`SingleUseSeal`] can't define seals by itself — and also knows nothing
//! about whether the seal is in fact closed: this requires a "seal medium": a
//! proof of publication medium on which the seals are defined.
//!
//! The module provides two options of implementing sch medium: synchronous
//! [`SealMedium`] and asynchronous `AsyncSealMedium`.
//!
//! ## Sample implementation
//!
//! Examples of implementations can be found in `bp::seals`.
//!
//! ## Further reading
//!
//! * Peter Todd. Preventing Consensus Fraud with Commitments and
//!   Single-Use-Seals.
//!   <https://petertodd.org/2016/commitments-and-single-use-seals>.
//! * Peter Todd. Scalable Semi-Trustless Asset Transfer via Single-Use-Seals
//!   and Proof-of-Publication. 1. Single-Use-Seal Definition.
//!   <https://petertodd.org/2017/scalable-single-use-seal-asset-transfer>

#[macro_use]
extern crate amplify_derive;
#[cfg(feature = "async")]
#[macro_use]
extern crate async_trait;

/// Single-use-seal trait: implement for a data structure that will hold a
/// single-use-seal definition and will contain a business logic for closing
/// seal over some message and verification of the seal against the message
/// and witness.
///
/// NB: It is recommended that single-use-seal instances to be instantiated
/// not by a constructor, but by a factory, i.e. "seal medium": data type
/// implementing either [`SealMedium`] or `AsyncSealMedium` traits.
#[cfg_attr(feature = "async", async_trait)]
pub trait SingleUseSeal {
    /// Associated type for the witness produced by the single-use-seal close
    /// procedure
    #[cfg(not(feature = "async"))]
    type Witness;

    /// Associated type for the witness produced by the single-use-seal close
    /// procedure
    #[cfg(feature = "async")]
    type Witness: Sync + Send;

    /// Type that contains seal definition
    type Definition;

    /// Message type that is supported by the current single-use-seal
    type Message: AsRef<[u8]>;

    /// Closing and verification errors
    type Error: std::error::Error;

    /// Closes seal over a message, producing *witness*
    ///
    /// NB: Closing of the seal MUST not change the internal state of the
    /// seal itself; all the data produced by the process must be placed
    /// into the returned Witness type
    fn close(&self, over: &Self::Message)
        -> Result<Self::Witness, Self::Error>;

    /// Verifies that the seal was indeed closed over the message on the
    /// specific seal medium (see [`SealMedium`])
    fn verify(
        &self,
        msg: &Self::Message,
        witness: &Self::Witness,
        medium: &impl SealMedium<Self>,
    ) -> Result<bool, Self::Error>
    where
        Self: Sized;

    /// Verifies that the seal was indeed closed over the message on the
    /// specific seal medium (see [`SealMedium`])
    #[cfg(feature = "async")]
    async fn verify_async(
        &self,
        msg: &Self::Message,
        witness: &Self::Witness,
        medium: &impl SealMediumAsync<Self>,
    ) -> Result<bool, Self::Error>
    where
        Self: Sized + Sync + Send;
}

/// Trait for proof-of-publication medium on which the seals are defined and
/// which can be used for convenience operations related to seals:
/// * finding out the seal status
/// * publishing witness information
/// * get some identifier on the exact place of the witness publication
/// * check validity of the witness publication identifier
///
/// Since the medium may require network communications or extensive computing
/// involved (like in case with blockchain) there is a special asynchronous
/// version of the seal medium `AsyncSealMedium`, which requires use of
/// `async` feature of this crate.
///
/// All these operations are medium-specific; for the same single-use-seal type
/// they may differ when are applied to different proof of publication mediums.
///
/// To read more on proof-of-publication please check
/// <https://petertodd.org/2014/setting-the-record-proof-of-publication>
pub trait SealMedium<Seal>
where
    Seal: SingleUseSeal,
{
    /// Publication id that may be used for referencing publication of
    /// witness data in the medium. By default set `()`, so [`SealMedium`]
    /// may not implement  publication id and related functions
    type PublicationId;

    /// Error type that contains reasons of medium access failure
    type Error: std::error::Error;

    /// Creates a single-use-seal having type of implementation-specific generic
    /// parameter `SEAL`.
    fn define_seal(
        &self,
        definition: &Seal::Definition,
    ) -> Result<Seal, Self::Error>;

    /// Checks the status for a given seal in proof-of-publication medium
    fn get_seal_status(&self, seal: &Seal) -> Result<SealStatus, Self::Error>;

    /// Publishes witness data to the medium. Function has default
    /// implementation doing nothing and returning
    /// [`SealMediumError::PublicationIdNotSupported`] error.
    fn publish_witness(
        &mut self,
        _witness: &Seal::Witness,
    ) -> Result<Self::PublicationId, SealMediumError<Self::Error>> {
        Err(SealMediumError::PublicationIdNotSupported)
    }

    /// Returns [`Self::PublicationId`] for a given witness, if any; the id is
    /// returned as an option. Function has default implementation doing
    /// nothing and just returning
    /// [`SealMediumError::PublicationIdNotSupported`] error.
    fn get_witness_publication_id(
        &self,
        _witness: &Seal::Witness,
    ) -> Result<Option<Self::PublicationId>, SealMediumError<Self::Error>> {
        Err(SealMediumError::PublicationIdNotSupported)
    }

    /// Validates whether a given publication id is present in the medium.
    /// Function has default implementation doing nothing and returning
    /// [`SealMediumError::PublicationIdNotSupported`] error.
    fn validate_publication_id(
        &self,
        _publication_id: &Self::PublicationId,
    ) -> Result<bool, SealMediumError<Self::Error>> {
        Err(SealMediumError::PublicationIdNotSupported)
    }
}

/// Asynchronous version of the [`SealMedium`] trait.
#[cfg(feature = "async")]
#[async_trait]
pub trait SealMediumAsync<Seal>
where
    Seal: SingleUseSeal + Sync + Send,
    Seal::Witness: Sync + Send,
    Self::PublicationId: Sync,
    Self: Send + Sync,
{
    /// Publication id that may be used for referencing publication of
    /// witness data in the medium. By default set `()`, so [`SealMedium`]
    /// may not implement  publication id and related functions
    type PublicationId;

    /// Error type that contains reasons of medium access failure
    type Error: std::error::Error;

    /// Creates a single-use-seal having type of implementation-specific generic
    /// parameter `SEAL`.
    async fn define_seal<D>(&self, definition: &D)
        -> Result<Seal, Self::Error>;

    /// Checks the status for a given seal in proof-of-publication medium
    async fn get_seal_status(
        &self,
        seal: &Seal,
    ) -> Result<SealStatus, Self::Error>;

    /// Publishes witness data to the medium. Function has default
    /// implementation doing nothing and returning
    /// [`SealMediumError::PublicationIdNotSupported`] error.
    async fn publish_witness(
        &mut self,
        _witness: &Seal::Witness,
    ) -> Result<Self::PublicationId, SealMediumError<Self::Error>>
    where
        Seal: 'async_trait,
    {
        Err(SealMediumError::PublicationIdNotSupported)
    }

    /// Returns [`Self::PublicationId`] for a given witness, if any; the id is
    /// returned as an option. Function has default implementation doing
    /// nothing and just returning
    /// [`SealMediumError::PublicationIdNotSupported`] error.
    async fn get_witness_publication_id(
        &self,
        _witness: &Seal::Witness,
    ) -> Result<Option<Self::PublicationId>, SealMediumError<Self::Error>>
    where
        Seal: 'async_trait,
    {
        Err(SealMediumError::PublicationIdNotSupported)
    }

    /// Validates whether a given publication id is present in the medium.
    /// Function has default implementation doing nothing and returning
    /// [`SealMediumError::PublicationIdNotSupported`] error.
    async fn validate_publication_id(
        &self,
        _publication_id: &Self::PublicationId,
    ) -> Result<bool, SealMediumError<Self::Error>>
    where
        Seal: 'async_trait,
    {
        Err(SealMediumError::PublicationIdNotSupported)
    }
}

/// Single-use-seal status returned by [`SealMedium::get_seal_status`] and
/// `AsyncSealMedium::get_seal_status` functions.
///
/// NB: It's important to note, that while its possible to deterministically
///   define was a given seal closed it yet may be not possible to find out
///   if the seal is open without provision of the message and witness; i.e.
///   seal status may be either "closed over message"
///   or "unknown". Some specific implementations of single-use-seals may define
///   procedure to deterministically prove that a given seal is not closed (i.e.
///   opened), however this is not a part of the specification and we should
///   not rely on the existence of such possibility in all cases.
#[derive(Clone, Copy, Debug, Display)]
#[display(Debug)]
#[repr(u8)]
pub enum SealStatus {
    /// It is unknown/undetermined whether the seal was closed
    Undefined = 0,

    /// The seal is closed
    Closed = 1,
}

/// Error returned by [`SealMedium`] and `AsyncSealMedium` functions related
/// to work with publication id ([`SealMedium::PublicationId`]). Required since
/// not all implementation of [`SealMedium`] may define publication identifier,
/// and the traits provide default implementation for these functions always
/// returning [`SealMediumError::PublicationIdNotSupported`]. If the
/// implementation would like to provide custom implementation, it may embed
/// standard error related to [`SealMedium`] operations within
/// [`SealMediumError::MediumAccessError`] case; the type of MediumAccessError
/// is defined through generic argument to [`SealMediumError`].
#[derive(Clone, Copy, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum SealMediumError<M: std::error::Error> {
    /// Can't access the publication medium
    #[from]
    MediumAccessError(M),

    /// Publication id is not supported
    PublicationIdNotSupported,
}
