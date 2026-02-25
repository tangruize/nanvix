// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Error Handling - Implementation
//!
//! Specification functions are in `lib.spec.rs` and proofs are in `lib.proof.rs`.

use vstd::prelude::*;

// Include specifications.
include!("lib.spec.rs");

// Include proofs (lemmas).
include!("lib.proof.rs");

//==================================================================================================
// Errno Constants
//==================================================================================================

// Verus note: these constants are defined in sysapi::errno in the source.
// Since sysapi is not available in the Verus verification crate, we inline
// the values here. They are identical to src/libs/sysapi/src/errno.rs.
type CInt = i32;
const EPERM: CInt = 1;
const ENOENT: CInt = 2;
const ESRCH: CInt = 3;
const EINTR: CInt = 4;
const EIO: CInt = 5;
const ENXIO: CInt = 6;
const E2BIG: CInt = 7;
const ENOEXEC: CInt = 8;
const EBADF: CInt = 9;
const ECHILD: CInt = 10;
const EAGAIN: CInt = 11;
const ENOMEM: CInt = 12;
const EACCES: CInt = 13;
const EFAULT: CInt = 14;
const ENOTBLK: CInt = 15;
const EBUSY: CInt = 16;
const EEXIST: CInt = 17;
const EXDEV: CInt = 18;
const ENODEV: CInt = 19;
const ENOTDIR: CInt = 20;
const EISDIR: CInt = 21;
const EINVAL: CInt = 22;
const ENFILE: CInt = 23;
const EMFILE: CInt = 24;
const ENOTTY: CInt = 25;
const ETXTBSY: CInt = 26;
const EFBIG: CInt = 27;
const ENOSPC: CInt = 28;
const ESPIPE: CInt = 29;
const EROFS: CInt = 30;
const EMLINK: CInt = 31;
const EPIPE: CInt = 32;
const EDOM: CInt = 33;
const ERANGE: CInt = 34;
const ENOMSG: CInt = 35;
const EIDRM: CInt = 36;
const ECHRNG: CInt = 37;
const EL2NSYNC: CInt = 38;
const EL3HLT: CInt = 39;
const EL3RST: CInt = 40;
const ELNRNG: CInt = 41;
const EUNATCH: CInt = 42;
const ENOCSI: CInt = 43;
const EL2HLT: CInt = 44;
const EDEADLK: CInt = 45;
const ENOLCK: CInt = 46;
const EBADE: CInt = 50;
const EBADR: CInt = 51;
const EXFULL: CInt = 52;
const ENOANO: CInt = 53;
const EBADRQC: CInt = 54;
const EBADSLT: CInt = 55;
const EDEADLOCK: CInt = 56;
const EBFONT: CInt = 57;
const ENOSTR: CInt = 60;
const ENODATA: CInt = 61;
const ETIME: CInt = 62;
const ENOSR: CInt = 63;
const ENONET: CInt = 64;
const ENOPKG: CInt = 65;
const EREMOTE: CInt = 66;
const ENOLINK: CInt = 67;
const EADV: CInt = 68;
const ESRMNT: CInt = 69;
const ECOMM: CInt = 70;
const EPROTO: CInt = 71;
const EMULTIHOP: CInt = 74;
const ELBIN: CInt = 75;
const EDOTDOT: CInt = 76;
const EBADMSG: CInt = 77;
const EFTYPE: CInt = 79;
const ENOTUNIQ: CInt = 80;
const EBADFD: CInt = 81;
const EREMCHG: CInt = 82;
const ELIBACC: CInt = 83;
const ELIBBAD: CInt = 84;
const ELIBSCN: CInt = 85;
const ELIBMAX: CInt = 86;
const ELIBEXEC: CInt = 87;
const ENOSYS: CInt = 88;
const ENOTEMPTY: CInt = 90;
const ENAMETOOLONG: CInt = 91;
const ELOOP: CInt = 92;
const EOPNOTSUPP: CInt = 95;
const EPFNOSUPPORT: CInt = 96;
const ECONNRESET: CInt = 104;
const ENOBUFS: CInt = 105;
const EAFNOSUPPORT: CInt = 106;
const EPROTOTYPE: CInt = 107;
const ENOTSOCK: CInt = 108;
const ENOPROTOOPT: CInt = 109;
const ESHUTDOWN: CInt = 110;
const ECONNREFUSED: CInt = 111;
const EADDRINUSE: CInt = 112;
const ECONNABORTED: CInt = 113;
const ENETUNREACH: CInt = 114;
const ENETDOWN: CInt = 115;
const ETIMEDOUT: CInt = 116;
const EHOSTDOWN: CInt = 117;
const EHOSTUNREACH: CInt = 118;
const EINPROGRESS: CInt = 119;
const EALREADY: CInt = 120;
const EDESTADDRREQ: CInt = 121;
const EMSGSIZE: CInt = 122;
const EPROTONOSUPPORT: CInt = 123;
const ESOCKTNOSUPPORT: CInt = 124;
const EADDRNOTAVAIL: CInt = 125;
const ENETRESET: CInt = 126;
const EISCONN: CInt = 127;
const ENOTCONN: CInt = 128;
const ETOOMANYREFS: CInt = 129;
const EUSERS: CInt = 131;
const EDQUOT: CInt = 132;
const ESTALE: CInt = 133;
const ENOTSUP: CInt = 134;
const ENOMEDIUM: CInt = 135;
const EILSEQ: CInt = 138;
const EOVERFLOW: CInt = 139;
const ECANCELED: CInt = 140;
const ENOTRECOVERABLE: CInt = 141;
const EOWNERDEAD: CInt = 142;
const ESTRPIPE: CInt = 143;

//==================================================================================================
// Structures
//==================================================================================================

verus! {

///
/// # Description
///
/// Error code for various adverse conditions.
///
/// # Notes
///
/// The values in this enumeration intentionally match the error codes defined in the Linux kernel.
///
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum ErrorCode {
    /// Operation not permitted.
    OperationNotPermitted = EPERM,
    /// No such file or directory.
    NoSuchEntry = ENOENT,
    /// No such process.
    NoSuchProcess = ESRCH,
    /// Interrupted system call.
    Interrupted = EINTR,
    /// I/O error.
    IoErr = EIO,
    /// No such device or address.
    NoSuchDeviceOrAddress = ENXIO,
    /// Argument list too long.
    TooBig = E2BIG,
    /// Exec format error.
    InvalidExecutableFormat = ENOEXEC,
    /// Bad file number.
    BadFile = EBADF,
    /// No child processes.
    NoChildProcess = ECHILD,
    /// Try again.
    TryAgain = EAGAIN,
    /// Out of memory.
    OutOfMemory = ENOMEM,
    /// Permission denied.
    PermissionDenied = EACCES,
    /// Bad address.
    BadAddress = EFAULT,
    /// Block device required.
    NotBlockDevice = ENOTBLK,
    /// Device or resource busy.
    ResourceBusy = EBUSY,
    /// File exists.
    EntryExists = EEXIST,
    /// Cross-device link.
    CrossDeviceLink = EXDEV,
    /// No such device.
    NoSuchDevice = ENODEV,
    /// Not a directory.
    InvalidDirectory = ENOTDIR,
    /// Is a directory.
    IsDirectory = EISDIR,
    /// Invalid argument.
    InvalidArgument = EINVAL,
    /// File table overflow.
    FileTableOVerflow = ENFILE,
    /// Too many open files.
    TooManyOpenFiles = EMFILE,
    /// Not a typewriter.
    NotTerminal = ENOTTY,
    /// Text file busy.
    TextFileBusy = ETXTBSY,
    /// File too large.
    FileTooLarge = EFBIG,
    /// No space left on device.
    NoSpaceOnDevice = ENOSPC,
    /// Illegal seek.
    IllegalSeek = ESPIPE,
    /// Read-only file system.
    ReadOnlyFileSystem = EROFS,
    /// Too many links.
    TooManyLinks = EMLINK,
    /// Broken pipe.
    BrokenPipe = EPIPE,
    /// Math argument out of domain of function.
    MathArgDomainErr = EDOM,
    /// Math result not representable.
    ValueOutOfRange = ERANGE,
    /// No message of desired type.
    NoMessageAvailable = ENOMSG,
    /// Identifier removed.
    IdentifierRemoved = EIDRM,
    /// Channel number out of range.
    OutOfRangeChannel = ECHRNG,
    /// Level 2 not synchronized.
    Level2NotSynchronized = EL2NSYNC,
    /// Level 3 halted.
    Level3Halted = EL3HLT,
    /// Level 3 reset.
    Level3Reset = EL3RST,
    /// Link number out of range.
    InvalidLinkNumber = ELNRNG,
    /// Protocol driver not attached.
    InvalidProtocolDriver = EUNATCH,
    /// No CSI structure available.
    NoStructAvailable = ENOCSI,
    /// Level 2 halted.
    Level2Halted = EL2HLT,
    /// Resource deadlock would occur.
    Deadlock = EDEADLK,
    /// No record locks available.
    LockNotAvailable = ENOLCK,
    /// Invalid exchange.
    InvalidExchange = EBADE,
    /// Invalid request descriptor.
    InvalidRequestDescriptor = EBADR,
    /// Exchange full.
    ExchangeFull = EXFULL,
    /// No anode.
    InvalidAnode = ENOANO,
    /// Invalid request code.
    InvalidRequestCode = EBADRQC,
    /// Invalid slot.
    InvalidSlot = EBADSLT,
    /// File locking deadlock error.
    DeadlockWouldOccur = EDEADLOCK,
    /// Bad font file format.
    BadFontFormat = EBFONT,
    /// Device not a stream.
    NoStreamDeviceAvailable = ENOSTR,
    /// No data available.
    NoDataAvailable = ENODATA,
    /// Timer expired.
    TimerExpired = ETIME,
    /// Out of streams resources.
    NoStreamResources = ENOSR,
    /// Machine is not on the network.
    NoNetwork = ENONET,
    /// Package not installed.
    MissingPackage = ENOPKG,
    /// Object is remote.
    RemoteObject = EREMOTE,
    /// Link has been severed.
    NoLink = ENOLINK,
    /// Advertise error.
    AdvertiseErr = EADV,
    /// Srmount error.
    MountErr = ESRMNT,
    /// Communication error on send.
    CommunicationErr = ECOMM,
    /// Protocol error.
    ProtocolErr = EPROTO,
    /// Multihop attempted.
    MultipleHopAttemped = EMULTIHOP,
    /// Remote inode.
    InodeRemote = ELBIN,
    /// RFS specific error.
    RfsErr = EDOTDOT,
    /// Not a data message.
    InvalidMessage = EBADMSG,
    /// Inappropriate file type or format.
    InvalidFileType = EFTYPE,
    /// Name not unique on network.
    NonUniqueName = ENOTUNIQ,
    /// File descriptor in bad state.
    InvalidFileDescriptor = EBADFD,
    /// Remote address changed.
    RemoteAddressChanged = EREMCHG,
    /// Can not access a needed shared library.
    LibraryAccessErr = ELIBACC,
    /// Accessing a corrupted shared library.
    InvalidLibraryAccess = ELIBBAD,
    /// .lib section in a.out corrupted.
    CorruptedLibSection = ELIBSCN,
    /// Attempting to link in too many shared libraries.
    ExcessiveLibraryLinkCount = ELIBMAX,
    /// Cannot exec a shared library directly.
    InvalidExecSharedLibrary = ELIBEXEC,
    /// Function not implemented.
    InvalidSysCall = ENOSYS,
    /// Directory not empty.
    DirectoryNotEmpty = ENOTEMPTY,
    /// File name too long.
    NameTooLong = ENAMETOOLONG,
    /// Too many symbolic links encountered.
    SymbolicLinkLoop = ELOOP,
    /// Operation not supported on socket.
    OperationNotSupportedOnSocket = EOPNOTSUPP,
    /// Protocol family not supported.
    ProtocolFamilyNotSupported = EPFNOSUPPORT,
    /// Connection reset by peer.
    ConnectionReset = ECONNRESET,
    /// No buffer space available.
    NoBufferSpace = ENOBUFS,
    /// Address family not supported by protocol.
    AddressFamilyNotSupported = EAFNOSUPPORT,
    /// Protocol wrong type for socket.
    BadProtocolType = EPROTOTYPE,
    /// Socket operation on non-socket.
    NotSocketFile = ENOTSOCK,
    /// Protocol not available.
    ProtocolOptionNotAvailable = ENOPROTOOPT,
    /// Cannot send after transport endpoint shutdown.
    TransportEndpointShutdown = ESHUTDOWN,
    /// Connection refused.
    ConnectionRefused = ECONNREFUSED,
    /// Address already in use.
    AddressInUse = EADDRINUSE,
    /// Software caused connection abort.
    ConnectionAborted = ECONNABORTED,
    /// Network is unreachable.
    NetworkUnreachable = ENETUNREACH,
    /// Network is down.
    NetworkDown = ENETDOWN,
    /// Connection timed out.
    OperationTimedOut = ETIMEDOUT,
    /// Host is down.
    HostDown = EHOSTDOWN,
    /// No route to host.
    HostUnreachable = EHOSTUNREACH,
    /// Operation now in progress.
    OperationInProgress = EINPROGRESS,
    /// Operation already in progress.
    OperationAlreadyInProgress = EALREADY,
    /// Destination address required.
    DestinationAddressRequired = EDESTADDRREQ,
    /// Message too long.
    MessageTooLong = EMSGSIZE,
    /// Protocol not supported.
    ProtocolNotSupported = EPROTONOSUPPORT,
    /// Socket type not supported.
    SocketTypeNotSupported = ESOCKTNOSUPPORT,
    /// Cannot assign requested address.
    AddressNotAvailable = EADDRNOTAVAIL,
    /// Network dropped connection on reset.
    NetworkReset = ENETRESET,
    /// Transport endpoint is already connected.
    TransportEndpointConnected = EISCONN,
    /// Transport endpoint is not connected.
    TransportEndpointNotConnected = ENOTCONN,
    /// Too many references: cannot splice.
    TooManyReferences = ETOOMANYREFS,
    /// Too many users.
    TooManyUsers = EUSERS,
    /// Disk quota exceeded.
    QuotaExceeded = EDQUOT,
    /// Stale file handle.
    StaleHandle = ESTALE,
    /// Operation not supported.
    OperationNotSupported = ENOTSUP,
    /// No medium found.
    MediumNotFound = ENOMEDIUM,
    /// Illegal byte sequence.
    IllegalByteSequence = EILSEQ,
    /// Value too large for defined data type.
    ValueOverflow = EOVERFLOW,
    /// Operation canceled.
    OperationCanceled = ECANCELED,
    /// State not recoverable.
    UnrecoverableState = ENOTRECOVERABLE,
    /// Owner died.
    DeadOwner = EOWNERDEAD,
    /// Streams pipe error.
    StreamPipeErr = ESTRPIPE,
}

} // verus!

impl ErrorCode {
    ///
    /// # Description
    ///
    /// Returns the error code as an `i32`.
    ///
    pub fn get(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for ErrorCode {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Error> {
        let value: i32 = if value < 0 {
            match value.checked_abs() {
                Some(abs) => abs,
                None => value,
            }
        } else {
            value
        };
        match value {
            EPERM => Ok(ErrorCode::OperationNotPermitted),
            ENOENT => Ok(ErrorCode::NoSuchEntry),
            ESRCH => Ok(ErrorCode::NoSuchProcess),
            EINTR => Ok(ErrorCode::Interrupted),
            EIO => Ok(ErrorCode::IoErr),
            ENXIO => Ok(ErrorCode::NoSuchDeviceOrAddress),
            E2BIG => Ok(ErrorCode::TooBig),
            ENOEXEC => Ok(ErrorCode::InvalidExecutableFormat),
            EBADF => Ok(ErrorCode::BadFile),
            ECHILD => Ok(ErrorCode::NoChildProcess),
            EAGAIN => Ok(ErrorCode::TryAgain),
            ENOMEM => Ok(ErrorCode::OutOfMemory),
            EACCES => Ok(ErrorCode::PermissionDenied),
            EFAULT => Ok(ErrorCode::BadAddress),
            ENOTBLK => Ok(ErrorCode::NotBlockDevice),
            EBUSY => Ok(ErrorCode::ResourceBusy),
            EEXIST => Ok(ErrorCode::EntryExists),
            EXDEV => Ok(ErrorCode::CrossDeviceLink),
            ENODEV => Ok(ErrorCode::NoSuchDevice),
            ENOTDIR => Ok(ErrorCode::InvalidDirectory),
            EISDIR => Ok(ErrorCode::IsDirectory),
            EINVAL => Ok(ErrorCode::InvalidArgument),
            ENFILE => Ok(ErrorCode::FileTableOVerflow),
            EMFILE => Ok(ErrorCode::TooManyOpenFiles),
            ENOTTY => Ok(ErrorCode::NotTerminal),
            ETXTBSY => Ok(ErrorCode::TextFileBusy),
            EFBIG => Ok(ErrorCode::FileTooLarge),
            ENOSPC => Ok(ErrorCode::NoSpaceOnDevice),
            ESPIPE => Ok(ErrorCode::IllegalSeek),
            EROFS => Ok(ErrorCode::ReadOnlyFileSystem),
            EMLINK => Ok(ErrorCode::TooManyLinks),
            EPIPE => Ok(ErrorCode::BrokenPipe),
            EDOM => Ok(ErrorCode::MathArgDomainErr),
            ERANGE => Ok(ErrorCode::ValueOutOfRange),
            ENOMSG => Ok(ErrorCode::NoMessageAvailable),
            EIDRM => Ok(ErrorCode::IdentifierRemoved),
            ECHRNG => Ok(ErrorCode::OutOfRangeChannel),
            EL2NSYNC => Ok(ErrorCode::Level2NotSynchronized),
            EL3HLT => Ok(ErrorCode::Level3Halted),
            EL3RST => Ok(ErrorCode::Level3Reset),
            ELNRNG => Ok(ErrorCode::InvalidLinkNumber),
            EUNATCH => Ok(ErrorCode::InvalidProtocolDriver),
            ENOCSI => Ok(ErrorCode::NoStructAvailable),
            EL2HLT => Ok(ErrorCode::Level2Halted),
            EDEADLK => Ok(ErrorCode::Deadlock),
            ENOLCK => Ok(ErrorCode::LockNotAvailable),
            EBADE => Ok(ErrorCode::InvalidExchange),
            EBADR => Ok(ErrorCode::InvalidRequestDescriptor),
            EXFULL => Ok(ErrorCode::ExchangeFull),
            ENOANO => Ok(ErrorCode::InvalidAnode),
            EBADRQC => Ok(ErrorCode::InvalidRequestCode),
            EBADSLT => Ok(ErrorCode::InvalidSlot),
            EDEADLOCK => Ok(ErrorCode::DeadlockWouldOccur),
            EBFONT => Ok(ErrorCode::BadFontFormat),
            ENOSTR => Ok(ErrorCode::NoStreamDeviceAvailable),
            ENODATA => Ok(ErrorCode::NoDataAvailable),
            ETIME => Ok(ErrorCode::TimerExpired),
            ENOSR => Ok(ErrorCode::NoStreamResources),
            ENONET => Ok(ErrorCode::NoNetwork),
            ENOPKG => Ok(ErrorCode::MissingPackage),
            EREMOTE => Ok(ErrorCode::RemoteObject),
            ENOLINK => Ok(ErrorCode::NoLink),
            EADV => Ok(ErrorCode::AdvertiseErr),
            ESRMNT => Ok(ErrorCode::MountErr),
            ECOMM => Ok(ErrorCode::CommunicationErr),
            EPROTO => Ok(ErrorCode::ProtocolErr),
            EMULTIHOP => Ok(ErrorCode::MultipleHopAttemped),
            ELBIN => Ok(ErrorCode::InodeRemote),
            EDOTDOT => Ok(ErrorCode::RfsErr),
            EBADMSG => Ok(ErrorCode::InvalidMessage),
            EFTYPE => Ok(ErrorCode::InvalidFileType),
            ENOTUNIQ => Ok(ErrorCode::NonUniqueName),
            EBADFD => Ok(ErrorCode::InvalidFileDescriptor),
            EREMCHG => Ok(ErrorCode::RemoteAddressChanged),
            ELIBACC => Ok(ErrorCode::LibraryAccessErr),
            ELIBBAD => Ok(ErrorCode::InvalidLibraryAccess),
            ELIBSCN => Ok(ErrorCode::CorruptedLibSection),
            ELIBMAX => Ok(ErrorCode::ExcessiveLibraryLinkCount),
            ELIBEXEC => Ok(ErrorCode::InvalidExecSharedLibrary),
            ENOSYS => Ok(ErrorCode::InvalidSysCall),
            ENOTEMPTY => Ok(ErrorCode::DirectoryNotEmpty),
            ENAMETOOLONG => Ok(ErrorCode::NameTooLong),
            ELOOP => Ok(ErrorCode::SymbolicLinkLoop),
            EOPNOTSUPP => Ok(ErrorCode::OperationNotSupportedOnSocket),
            EPFNOSUPPORT => Ok(ErrorCode::ProtocolFamilyNotSupported),
            ECONNRESET => Ok(ErrorCode::ConnectionReset),
            ENOBUFS => Ok(ErrorCode::NoBufferSpace),
            EAFNOSUPPORT => Ok(ErrorCode::AddressFamilyNotSupported),
            EPROTOTYPE => Ok(ErrorCode::BadProtocolType),
            ENOTSOCK => Ok(ErrorCode::NotSocketFile),
            ENOPROTOOPT => Ok(ErrorCode::ProtocolOptionNotAvailable),
            ESHUTDOWN => Ok(ErrorCode::TransportEndpointShutdown),
            ECONNREFUSED => Ok(ErrorCode::ConnectionRefused),
            EADDRINUSE => Ok(ErrorCode::AddressInUse),
            ECONNABORTED => Ok(ErrorCode::ConnectionAborted),
            ENETUNREACH => Ok(ErrorCode::NetworkUnreachable),
            ENETDOWN => Ok(ErrorCode::NetworkDown),
            ETIMEDOUT => Ok(ErrorCode::OperationTimedOut),
            EHOSTDOWN => Ok(ErrorCode::HostDown),
            EHOSTUNREACH => Ok(ErrorCode::HostUnreachable),
            EINPROGRESS => Ok(ErrorCode::OperationInProgress),
            EALREADY => Ok(ErrorCode::OperationAlreadyInProgress),
            EDESTADDRREQ => Ok(ErrorCode::DestinationAddressRequired),
            EMSGSIZE => Ok(ErrorCode::MessageTooLong),
            EPROTONOSUPPORT => Ok(ErrorCode::ProtocolNotSupported),
            ESOCKTNOSUPPORT => Ok(ErrorCode::SocketTypeNotSupported),
            EADDRNOTAVAIL => Ok(ErrorCode::AddressNotAvailable),
            ENETRESET => Ok(ErrorCode::NetworkReset),
            EISCONN => Ok(ErrorCode::TransportEndpointConnected),
            ENOTCONN => Ok(ErrorCode::TransportEndpointNotConnected),
            ETOOMANYREFS => Ok(ErrorCode::TooManyReferences),
            EUSERS => Ok(ErrorCode::TooManyUsers),
            EDQUOT => Ok(ErrorCode::QuotaExceeded),
            ESTALE => Ok(ErrorCode::StaleHandle),
            ENOTSUP => Ok(ErrorCode::OperationNotSupported),
            ENOMEDIUM => Ok(ErrorCode::MediumNotFound),
            EILSEQ => Ok(ErrorCode::IllegalByteSequence),
            EOVERFLOW => Ok(ErrorCode::ValueOverflow),
            ECANCELED => Ok(ErrorCode::OperationCanceled),
            ENOTRECOVERABLE => Ok(ErrorCode::UnrecoverableState),
            EOWNERDEAD => Ok(ErrorCode::DeadOwner),
            ESTRPIPE => Ok(ErrorCode::StreamPipeErr),
            _ => Err(invalid_error_code(value)),
        }
    }
}

///
/// # Description
///
/// Constructs a default error.
///
/// # Parameters
///
/// - `value`: Error code (unused).
///
/// # Returns
///
/// Default error.
///
fn invalid_error_code(_value: i32) -> Error {
    Error {
        code: ErrorCode::InvalidArgument,
        reason: "invalid error code",
    }
}

//==================================================================================================
// Verified Structures and Functions
//==================================================================================================

verus! {

#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub reason: &'static str,
}

impl Error {
    #[verifier::external_body]
    pub fn new(code: ErrorCode, reason: &'static str) -> (result: Error)
        ensures
            result.code == code,
            result.reason == reason,
    {
        Error { code, reason }
    }

    /// Logs an error message.
    ///
    /// # Note
    ///
    /// Verus does not support the `error!` macro. This method is used by
    /// verified callers (e.g., frame allocator) as a replacement.
    #[verifier::external_body]
    pub fn log(&self) {
        eprintln!("error: {:?}: {}", self.code, self.reason);
    }
}

} // verus!

//==================================================================================================
// Trait Implementations
//==================================================================================================

impl core::error::Error for ErrorCode {}

impl core::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "error={self:?}")
    }
}

impl From<ErrorCode> for u32 {
    fn from(errno: ErrorCode) -> Self {
        errno as u32
    }
}

impl From<ErrorCode> for i32 {
    fn from(errno: ErrorCode) -> Self {
        errno as i32
    }
}

impl From<ErrorCode> for i64 {
    fn from(errno: ErrorCode) -> Self {
        errno as i64
    }
}

impl From<ErrorCode> for i16 {
    fn from(errno: ErrorCode) -> Self {
        errno as i16
    }
}

impl From<ErrorCode> for u16 {
    fn from(errno: ErrorCode) -> Self {
        errno as u16
    }
}

impl TryFrom<i64> for ErrorCode {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value: i32 = value
            .try_into()
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid error code"))?;
        ErrorCode::try_from(value)
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid error code"))
    }
}
