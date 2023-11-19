use const_oid::{AssociatedOid, ObjectIdentifier};
use digest::{Digest, DynDigest};

/// [`get_digest`] would lookup the
pub fn get_digest(identifier: ObjectIdentifier) -> Option<Box<dyn DynDigest>> {
    macro_rules! check_match {
        ($h:ty) => {
            if (identifier == <$h as AssociatedOid>::OID) {
                return Some(Box::new(<$h>::new()));
            }
        };
    }

    #[cfg(feature = "sha1")]
    {
        check_match!(sha1::Sha1);
    }

    #[cfg(feature = "sha2")]
    {
        check_match!(sha2::Sha224);
        check_match!(sha2::Sha256);
        check_match!(sha2::Sha384);
        check_match!(sha2::Sha512);
    }

    #[cfg(feature = "sha3")]
    {
        check_match!(sha3::Sha3_224);
        check_match!(sha3::Sha3_256);
        check_match!(sha3::Sha3_384);
        check_match!(sha3::Sha3_512);
    }

    None
}
