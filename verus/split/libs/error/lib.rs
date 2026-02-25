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

verus! {

//==================================================================================================
// Structures
//==================================================================================================

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
    OperationNotPermitted = 1,
    /// No such file or directory.
    NoSuchEntry = 2,
    /// No such process.
    NoSuchProcess = 3,
    /// Interrupted system call.
    Interrupted = 4,
    /// I/O error.
    IoErr = 5,
    /// No such device or address.
    NoSuchDeviceOrAddress = 6,
    /// Argument list too long.
    TooBig = 7,
    /// Exec format error.
    InvalidExecutableFormat = 8,
    /// Bad file number.
    BadFile = 9,
    /// No child processes.
    NoChildProcess = 10,
    /// Try again.
    TryAgain = 11,
    /// Out of memory.
    OutOfMemory = 12,
    /// Permission denied.
    PermissionDenied = 13,
    /// Bad address.
    BadAddress = 14,
    /// Block device required.
    NotBlockDevice = 15,
    /// Device or resource busy.
    ResourceBusy = 16,
    /// File exists.
    EntryExists = 17,
    /// Cross-device link.
    CrossDeviceLink = 18,
    /// No such device.
    NoSuchDevice = 19,
    /// Not a directory.
    InvalidDirectory = 20,
    /// Is a directory.
    IsDirectory = 21,
    /// Invalid argument.
    InvalidArgument = 22,
    /// File table overflow.
    FileTableOVerflow = 23,
    /// Too many open files.
    TooManyOpenFiles = 24,
    /// Not a typewriter.
    NotTerminal = 25,
    /// Text file busy.
    TextFileBusy = 26,
    /// File too large.
    FileTooLarge = 27,
    /// No space left on device.
    NoSpaceOnDevice = 28,
    /// Illegal seek.
    IllegalSeek = 29,
    /// Read-only file system.
    ReadOnlyFileSystem = 30,
    /// Too many links.
    TooManyLinks = 31,
    /// Broken pipe.
    BrokenPipe = 32,
    /// Math argument out of domain of function.
    MathArgDomainErr = 33,
    /// Math result not representable.
    ValueOutOfRange = 34,
    /// No message of desired type.
    NoMessageAvailable = 35,
    /// Identifier removed.
    IdentifierRemoved = 36,
    /// Channel number out of range.
    OutOfRangeChannel = 37,
    /// Level 2 not synchronized.
    Level2NotSynchronized = 38,
    /// Level 3 halted.
    Level3Halted = 39,
    /// Level 3 reset.
    Level3Reset = 40,
    /// Link number out of range.
    InvalidLinkNumber = 41,
    /// Protocol driver not attached.
    InvalidProtocolDriver = 42,
    /// No CSI structure available.
    NoStructAvailable = 43,
    /// Level 2 halted.
    Level2Halted = 44,
    /// Resource deadlock would occur.
    Deadlock = 45,
    /// No record locks available.
    LockNotAvailable = 46,
    /// Invalid exchange.
    InvalidExchange = 50,
    /// Invalid request descriptor.
    InvalidRequestDescriptor = 51,
    /// Exchange full.
    ExchangeFull = 52,
    /// No anode.
    InvalidAnode = 53,
    /// Invalid request code.
    InvalidRequestCode = 54,
    /// Invalid slot.
    InvalidSlot = 55,
    /// File locking deadlock error.
    DeadlockWouldOccur = 56,
    /// Bad font file format.
    BadFontFormat = 57,
    /// Device not a stream.
    NoStreamDeviceAvailable = 60,
    /// No data available.
    NoDataAvailable = 61,
    /// Timer expired.
    TimerExpired = 62,
    /// Out of streams resources.
    NoStreamResources = 63,
    /// Machine is not on the network.
    NoNetwork = 64,
    /// Package not installed.
    MissingPackage = 65,
    /// Object is remote.
    RemoteObject = 66,
    /// Link has been severed.
    NoLink = 67,
    /// Advertise error.
    AdvertiseErr = 68,
    /// Srmount error.
    MountErr = 69,
    /// Communication error on send.
    CommunicationErr = 70,
    /// Protocol error.
    ProtocolErr = 71,
    /// Multihop attempted.
    MultipleHopAttemped = 74,
    /// Remote inode.
    InodeRemote = 75,
    /// RFS specific error.
    RfsErr = 76,
    /// Not a data message.
    InvalidMessage = 77,
    /// Inappropriate file type or format.
    InvalidFileType = 79,
    /// Name not unique on network.
    NonUniqueName = 80,
    /// File descriptor in bad state.
    InvalidFileDescriptor = 81,
    /// Remote address changed.
    RemoteAddressChanged = 82,
    /// Can not access a needed shared library.
    LibraryAccessErr = 83,
    /// Accessing a corrupted shared library.
    InvalidLibraryAccess = 84,
    /// .lib section in a.out corrupted.
    CorruptedLibSection = 85,
    /// Attempting to link in too many shared libraries.
    ExcessiveLibraryLinkCount = 86,
    /// Cannot exec a shared library directly.
    InvalidExecSharedLibrary = 87,
    /// Function not implemented.
    InvalidSysCall = 88,
    /// Directory not empty.
    DirectoryNotEmpty = 90,
    /// File name too long.
    NameTooLong = 91,
    /// Too many symbolic links encountered.
    SymbolicLinkLoop = 92,
    /// Operation not supported on socket.
    OperationNotSupportedOnSocket = 95,
    /// Protocol family not supported.
    ProtocolFamilyNotSupported = 96,
    /// Connection reset by peer.
    ConnectionReset = 104,
    /// No buffer space available.
    NoBufferSpace = 105,
    /// Address family not supported by protocol.
    AddressFamilyNotSupported = 106,
    /// Protocol wrong type for socket.
    BadProtocolType = 107,
    /// Socket operation on non-socket.
    NotSocketFile = 108,
    /// Protocol not available.
    ProtocolOptionNotAvailable = 109,
    /// Cannot send after transport endpoint shutdown.
    TransportEndpointShutdown = 110,
    /// Connection refused.
    ConnectionRefused = 111,
    /// Address already in use.
    AddressInUse = 112,
    /// Software caused connection abort.
    ConnectionAborted = 113,
    /// Network is unreachable.
    NetworkUnreachable = 114,
    /// Network is down.
    NetworkDown = 115,
    /// Connection timed out.
    OperationTimedOut = 116,
    /// Host is down.
    HostDown = 117,
    /// No route to host.
    HostUnreachable = 118,
    /// Operation now in progress.
    OperationInProgress = 119,
    /// Operation already in progress.
    OperationAlreadyInProgress = 120,
    /// Destination address required.
    DestinationAddressRequired = 121,
    /// Message too long.
    MessageTooLong = 122,
    /// Protocol not supported.
    ProtocolNotSupported = 123,
    /// Socket type not supported.
    SocketTypeNotSupported = 124,
    /// Cannot assign requested address.
    AddressNotAvailable = 125,
    /// Network dropped connection on reset.
    NetworkReset = 126,
    /// Transport endpoint is already connected.
    TransportEndpointConnected = 127,
    /// Transport endpoint is not connected.
    TransportEndpointNotConnected = 128,
    /// Too many references: cannot splice.
    TooManyReferences = 129,
    /// Too many users.
    TooManyUsers = 131,
    /// Disk quota exceeded.
    QuotaExceeded = 132,
    /// Stale file handle.
    StaleHandle = 133,
    /// Operation not supported.
    OperationNotSupported = 134,
    /// No medium found.
    MediumNotFound = 135,
    /// Illegal byte sequence.
    IllegalByteSequence = 138,
    /// Value too large for defined data type.
    ValueOverflow = 139,
    /// Operation canceled.
    OperationCanceled = 140,
    /// State not recoverable.
    UnrecoverableState = 141,
    /// Owner died.
    DeadOwner = 142,
    /// Streams pipe error.
    StreamPipeErr = 143,
}

#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub reason: &'static str,
}

//==================================================================================================
// Implementation
//==================================================================================================

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

impl Error {
    pub fn new(code: ErrorCode, reason: &'static str) -> (result: Self)
        ensures
            result.code == code,
            result.reason == reason,
    {
        Self { code, reason }
    }

    // Verus note: kept because it is called by other verified modules (e.g., frame.rs).
    // Not present in the original source but required for Verus codebase compilation.
    #[verifier::external_body]
    pub fn log(&self) {
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
fn invalid_error_code(_value: i32) -> (result: Error)
    ensures
        result.code == ErrorCode::InvalidArgument,
        result.reason == "invalid error code",
{
    Error {
        code: ErrorCode::InvalidArgument,
        reason: "invalid error code",
    }
}

} // verus!

//==================================================================================================
// Trait Implementations (outside verus! — Verus cannot verify trait impls directly)
//==================================================================================================

#[verifier::external]
impl core::error::Error for ErrorCode {}

#[verifier::external]
impl core::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "error={self:?}")
    }
}

#[verifier::external]
impl From<ErrorCode> for u32 {
    fn from(errno: ErrorCode) -> Self {
        errno as u32
    }
}

#[verifier::external]
impl From<ErrorCode> for i32 {
    fn from(errno: ErrorCode) -> Self {
        errno as i32
    }
}

#[verifier::external]
impl From<ErrorCode> for i64 {
    fn from(errno: ErrorCode) -> Self {
        errno as i64
    }
}

#[verifier::external]
impl From<ErrorCode> for i16 {
    fn from(errno: ErrorCode) -> Self {
        errno as i16
    }
}

#[verifier::external]
impl From<ErrorCode> for u16 {
    fn from(errno: ErrorCode) -> Self {
        errno as u16
    }
}

// Verus note: TryFrom<i32> placed outside verus! because trait impls are not supported inside.
// Uses hardcoded i32 values instead of sysapi::errno constants (not available in Verus crate).
#[verifier::external]
impl TryFrom<i32> for ErrorCode {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Error> {
        // Normalize to a positive errno value when possible, avoiding overflow on i32::MIN.
        let value: i32 = if value < 0 {
            match value.checked_abs() {
                Some(abs) => abs,
                None => value,
            }
        } else {
            value
        };
        match value {
            1 => Ok(ErrorCode::OperationNotPermitted),
            2 => Ok(ErrorCode::NoSuchEntry),
            3 => Ok(ErrorCode::NoSuchProcess),
            4 => Ok(ErrorCode::Interrupted),
            5 => Ok(ErrorCode::IoErr),
            6 => Ok(ErrorCode::NoSuchDeviceOrAddress),
            7 => Ok(ErrorCode::TooBig),
            8 => Ok(ErrorCode::InvalidExecutableFormat),
            9 => Ok(ErrorCode::BadFile),
            10 => Ok(ErrorCode::NoChildProcess),
            11 => Ok(ErrorCode::TryAgain),
            12 => Ok(ErrorCode::OutOfMemory),
            13 => Ok(ErrorCode::PermissionDenied),
            14 => Ok(ErrorCode::BadAddress),
            15 => Ok(ErrorCode::NotBlockDevice),
            16 => Ok(ErrorCode::ResourceBusy),
            17 => Ok(ErrorCode::EntryExists),
            18 => Ok(ErrorCode::CrossDeviceLink),
            19 => Ok(ErrorCode::NoSuchDevice),
            20 => Ok(ErrorCode::InvalidDirectory),
            21 => Ok(ErrorCode::IsDirectory),
            22 => Ok(ErrorCode::InvalidArgument),
            23 => Ok(ErrorCode::FileTableOVerflow),
            24 => Ok(ErrorCode::TooManyOpenFiles),
            25 => Ok(ErrorCode::NotTerminal),
            26 => Ok(ErrorCode::TextFileBusy),
            27 => Ok(ErrorCode::FileTooLarge),
            28 => Ok(ErrorCode::NoSpaceOnDevice),
            29 => Ok(ErrorCode::IllegalSeek),
            30 => Ok(ErrorCode::ReadOnlyFileSystem),
            31 => Ok(ErrorCode::TooManyLinks),
            32 => Ok(ErrorCode::BrokenPipe),
            33 => Ok(ErrorCode::MathArgDomainErr),
            34 => Ok(ErrorCode::ValueOutOfRange),
            35 => Ok(ErrorCode::NoMessageAvailable),
            36 => Ok(ErrorCode::IdentifierRemoved),
            37 => Ok(ErrorCode::OutOfRangeChannel),
            38 => Ok(ErrorCode::Level2NotSynchronized),
            39 => Ok(ErrorCode::Level3Halted),
            40 => Ok(ErrorCode::Level3Reset),
            41 => Ok(ErrorCode::InvalidLinkNumber),
            42 => Ok(ErrorCode::InvalidProtocolDriver),
            43 => Ok(ErrorCode::NoStructAvailable),
            44 => Ok(ErrorCode::Level2Halted),
            45 => Ok(ErrorCode::Deadlock),
            46 => Ok(ErrorCode::LockNotAvailable),
            50 => Ok(ErrorCode::InvalidExchange),
            51 => Ok(ErrorCode::InvalidRequestDescriptor),
            52 => Ok(ErrorCode::ExchangeFull),
            53 => Ok(ErrorCode::InvalidAnode),
            54 => Ok(ErrorCode::InvalidRequestCode),
            55 => Ok(ErrorCode::InvalidSlot),
            56 => Ok(ErrorCode::DeadlockWouldOccur),
            57 => Ok(ErrorCode::BadFontFormat),
            60 => Ok(ErrorCode::NoStreamDeviceAvailable),
            61 => Ok(ErrorCode::NoDataAvailable),
            62 => Ok(ErrorCode::TimerExpired),
            63 => Ok(ErrorCode::NoStreamResources),
            64 => Ok(ErrorCode::NoNetwork),
            65 => Ok(ErrorCode::MissingPackage),
            66 => Ok(ErrorCode::RemoteObject),
            67 => Ok(ErrorCode::NoLink),
            68 => Ok(ErrorCode::AdvertiseErr),
            69 => Ok(ErrorCode::MountErr),
            70 => Ok(ErrorCode::CommunicationErr),
            71 => Ok(ErrorCode::ProtocolErr),
            74 => Ok(ErrorCode::MultipleHopAttemped),
            75 => Ok(ErrorCode::InodeRemote),
            76 => Ok(ErrorCode::RfsErr),
            77 => Ok(ErrorCode::InvalidMessage),
            79 => Ok(ErrorCode::InvalidFileType),
            80 => Ok(ErrorCode::NonUniqueName),
            81 => Ok(ErrorCode::InvalidFileDescriptor),
            82 => Ok(ErrorCode::RemoteAddressChanged),
            83 => Ok(ErrorCode::LibraryAccessErr),
            84 => Ok(ErrorCode::InvalidLibraryAccess),
            85 => Ok(ErrorCode::CorruptedLibSection),
            86 => Ok(ErrorCode::ExcessiveLibraryLinkCount),
            87 => Ok(ErrorCode::InvalidExecSharedLibrary),
            88 => Ok(ErrorCode::InvalidSysCall),
            90 => Ok(ErrorCode::DirectoryNotEmpty),
            91 => Ok(ErrorCode::NameTooLong),
            92 => Ok(ErrorCode::SymbolicLinkLoop),
            95 => Ok(ErrorCode::OperationNotSupportedOnSocket),
            96 => Ok(ErrorCode::ProtocolFamilyNotSupported),
            104 => Ok(ErrorCode::ConnectionReset),
            105 => Ok(ErrorCode::NoBufferSpace),
            106 => Ok(ErrorCode::AddressFamilyNotSupported),
            107 => Ok(ErrorCode::BadProtocolType),
            108 => Ok(ErrorCode::NotSocketFile),
            109 => Ok(ErrorCode::ProtocolOptionNotAvailable),
            110 => Ok(ErrorCode::TransportEndpointShutdown),
            111 => Ok(ErrorCode::ConnectionRefused),
            112 => Ok(ErrorCode::AddressInUse),
            113 => Ok(ErrorCode::ConnectionAborted),
            114 => Ok(ErrorCode::NetworkUnreachable),
            115 => Ok(ErrorCode::NetworkDown),
            116 => Ok(ErrorCode::OperationTimedOut),
            117 => Ok(ErrorCode::HostDown),
            118 => Ok(ErrorCode::HostUnreachable),
            119 => Ok(ErrorCode::OperationInProgress),
            120 => Ok(ErrorCode::OperationAlreadyInProgress),
            121 => Ok(ErrorCode::DestinationAddressRequired),
            122 => Ok(ErrorCode::MessageTooLong),
            123 => Ok(ErrorCode::ProtocolNotSupported),
            124 => Ok(ErrorCode::SocketTypeNotSupported),
            125 => Ok(ErrorCode::AddressNotAvailable),
            126 => Ok(ErrorCode::NetworkReset),
            127 => Ok(ErrorCode::TransportEndpointConnected),
            128 => Ok(ErrorCode::TransportEndpointNotConnected),
            129 => Ok(ErrorCode::TooManyReferences),
            131 => Ok(ErrorCode::TooManyUsers),
            132 => Ok(ErrorCode::QuotaExceeded),
            133 => Ok(ErrorCode::StaleHandle),
            134 => Ok(ErrorCode::OperationNotSupported),
            135 => Ok(ErrorCode::MediumNotFound),
            138 => Ok(ErrorCode::IllegalByteSequence),
            139 => Ok(ErrorCode::ValueOverflow),
            140 => Ok(ErrorCode::OperationCanceled),
            141 => Ok(ErrorCode::UnrecoverableState),
            142 => Ok(ErrorCode::DeadOwner),
            143 => Ok(ErrorCode::StreamPipeErr),
            _ => Err(invalid_error_code(value)),
        }
    }
}

#[verifier::external]
impl TryFrom<i64> for ErrorCode {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        // Attempt to convert i64 to i32.
        let value: i32 = value
            .try_into()
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid error code"))?;

        // Attempt to convert i32 to ErrorCode.
        ErrorCode::try_from(value)
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid error code"))
    }
}
