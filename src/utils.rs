use super::bindings;
use super::utils::FiftyOneDegreesError::{
    AssertionError, CStringCreationError, InternalApiError,
};
use std::ffi::CString;
use std::path::Path;
use strum_macros::{AsRefStr, Display};
use thiserror::Error;

#[derive(Debug, Display)]
pub(crate) enum CStringKind {
    #[strum(serialize = "evidence key")]
    FilePath,
    #[strum(serialize = "evidence value")]
    EvidenceKey,
    #[strum(serialize = "evidence value")]
    EvidenceValue,
    #[strum(serialize = "property name")]
    PropertyName,
    #[strum(serialize = "hash result separator")]
    HashResultSeparator,
}

#[derive(Debug, Display)]
pub(crate) enum Operation {
    #[strum(serialize = "read data file")]
    ReadDataFile,
    #[strum(serialize = "initialize manager")]
    InitManager,
    #[strum(serialize = "create evidence")]
    CreateEvidence,
    #[strum(serialize = "apply evidence")]
    ApplyEvidence,
    #[strum(serialize = "read property")]
    ReadProperty,
}

#[derive(Debug, Display, AsRefStr)]
pub(crate) enum ReadFileError {
    #[strum(serialize = "file does not exist")]
    NotExists,
    #[strum(serialize = "is not a file")]
    IsNotFile,
}

#[derive(Debug, Error)]
pub(crate) enum FiftyOneDegreesError {
    #[error("CString creation error for: {0}")]
    CStringCreationError(CStringKind),
    #[error(
        "FiftyOneDegrees internal API error for operation: {0}, status code: {1}, message: {2}, error: {3})"
    )]
    InternalApiError(Operation, u32, &'static str, &'static str),
    #[error("FiftyOneDegrees unsafe operation error: {0}")]
    UnsafeOperationError(String),
    #[error("FiftyOneDegrees assertion error for operation {0}: {1}")]
    AssertionError(Operation, &'static str),
    #[error("FiftyOneDegrees IO error: {0}, cause: {1:?}")]
    IOError(&'static str, Option<std::io::Error>),
}

impl FiftyOneDegreesError {
    fn new_read_file_assertion_error(error: &'static ReadFileError) -> Self {
        AssertionError(Operation::ReadDataFile, error.as_ref())
    }
}

pub(crate) type FiftyOneDegreesResult<T> = Result<T, FiftyOneDegreesError>;

pub(crate) fn build_cstring(kind: CStringKind, str: &str) -> FiftyOneDegreesResult<CString> {
    CString::new(str).map_err(|_| CStringCreationError(kind))
}

pub(crate) fn status_to_error_message(status: u32) -> &'static str {
    match status {
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_SUCCESS  => "Success",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_INSUFFICIENT_MEMORY => "Lack of memory",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_CORRUPT_DATA => "Corrupt data",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_INCORRECT_VERSION => "Incorrect version",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_NOT_FOUND => "File not found",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_BUSY => "File busy",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_FAILURE => "File failure",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_NOT_SET => "Not set (should never be returned)",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_POINTER_OUT_OF_BOUNDS => "Pointer out of bounds",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_NULL_POINTER => "Null pointer",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_TOO_MANY_OPEN_FILES => "Too many open files",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_REQ_PROP_NOT_PRESENT => "Required property not present",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_PROFILE_EMPTY => "Profile is empty",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_COLLECTION_FAILURE => "Collection failure",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_COPY_ERROR => "File copy error",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_EXISTS_ERROR => "File exists error",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_WRITE_ERROR => "File write error",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_READ_ERROR => "File read error",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_PERMISSION_DENIED => "File permission denied",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_PATH_TOO_LONG => "File path too long",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_END_OF_DOCUMENT => "File end of document",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_END_OF_DOCUMENTS => "File end of documents",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_FILE_END_OF_FILE => "File end of file",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_ENCODING_ERROR => "Encoding error",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_INVALID_COLLECTION_CONFIG => "Invalid collection config",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_INVALID_CONFIG => "Invalid config",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_INSUFFICIENT_HANDLES => "Insufficient handles",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_COLLECTION_INDEX_OUT_OF_RANGE => "Collection index out of range",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_COLLECTION_OFFSET_OUT_OF_RANGE => "Collection offset out of range",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_COLLECTION_FILE_SEEK_FAIL => "Collection file seek fail",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_COLLECTION_FILE_READ_FAIL => "Collection file read fail",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_INCORRECT_IP_ADDRESS_FORMAT => "Incorrect IP address format",
        bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_TEMP_FILE_ERROR => "Temp file error",
        _ => "Unknown error"
    }
}

pub(crate) fn ger_error_msg(exception: *mut bindings::fiftyoneDegreesException) -> &'static str {
    unsafe {
        if exception.is_null() {
            return "No exception available";
        }
        let msg_ptr = bindings::fiftyoneDegreesExceptionGetMessage(exception);
        let message = if !msg_ptr.is_null() {
            std::ffi::CStr::from_ptr(msg_ptr)
                .to_str()
                .unwrap_or("Unknown error")
        } else {
            "No error message available"
        };
        message
    }
}

pub(crate) fn verify_exception(
    exception: *mut bindings::fiftyoneDegreesException,
    operation: Operation,
) -> FiftyOneDegreesResult<()> {
    if !exception.is_null() {
        let status = unsafe { *exception }.status;
        if status != bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_SUCCESS {
            return Err(InternalApiError(
                operation,
                status,
                status_to_error_message(status),
                ger_error_msg(exception),
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_data_file_path(path: &Path) -> FiftyOneDegreesResult<()> {
    if !path.exists() {
        return Err(FiftyOneDegreesError::new_read_file_assertion_error(
            &ReadFileError::NotExists,
        ));
    }
    if !path.is_file() {
        return Err(FiftyOneDegreesError::new_read_file_assertion_error(
            &ReadFileError::IsNotFile,
        ));
    }
    Ok(())
}
