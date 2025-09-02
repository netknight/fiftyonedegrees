use super::bindings;
use super::utils::{
    build_cstring, status_to_error_message, verify_data_file_path, verify_exception, CStringKind,
    FiftyOneDegreesResult, Operation,
};
use super::utils::FiftyOneDegreesError::{
    AssertionError, IOError, InternalApiError, UnsafeOperationError,
};
use itertools::Itertools;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr::null_mut;
use strum_macros::{AsRefStr, Display};

#[derive(Debug, Clone, Copy, PartialEq, Display, AsRefStr)]
pub enum PropertyName {
    // Device info
    DeviceId,
    DeviceType,
    CrawlerName,
    // Device properties
    HasTouchScreen,
    IsScreenFoldable,
    IsSmallScreen,
    IsEmailBrowser,
    IsEmulatingDesktop,
    IsEmulatingDevice,
    IsWebApp,
    IsConsole,
    IsEReader,
    IsMediaHub,
    IsMobile,
    IsSmartWatch,
    IsTablet,
    IsTv,
    IsCrawler,
    IsArtificialIntelligence,
    // Device native info
    NativeBrand,
    NativeDevice,
    NativeModel,
    NativeName,
    NativePlatform,

    // Browser info
    BrowserFamily,
    BrowserName,
    BrowserVendor,
    BrowserVersion,
    BrowserReleaseYear,
    BrowserSourceProject,
    BrowserSourceProjectVersion,
    BrowserRank,

    // Browser options
    Canvas,
    CookiesCapable,
    CssCanvas,
    DeviceOrientation,
    Fetch,
    Fullscreen,
    GeoLocation,
    IndexedDB,
    InVRMode,
    Javascript,
    Viewport,

    // Platform info
    PlatformName,
    PlatformVendor,
    PlatformVersion,
    PlatformReleaseYear,
    PlatformRank,

    // Hardware info
    HardwareName,
    HardwareVendor,
    HardwareFamily,
    HardwareModel,
    HardwareModelVariants,
    HardwareCarrier,
    HardwareRank,
    OEM,         // Indicates the name of the company that manufactures the device.
    ReleaseYear, // Indicates the year in which the device was released or the year in which the device was first seen by 51Degrees (if the release date cannot be identified).
    // Hardware Screen info
    BitsPerPixel,
    PixelRatio,
    ScreenInchesDiagonal,
    ScreenPixelsHeight,
    ScreenPixelsPhysicalHeight,
    ScreenPixelsPhysicalWidth,
    ScreenPixelsWidth,
    ScreenType,
    // Hardware Network
    RegisteredCountry,
    RegisteredName,
    RegisteredOwner,

    // Other
    Profiles,
    Popularity, // Refers to the number of unique client IPs from which this device has been seen.
    PriceBand, // Indicates a price range describing the recommended retail price of the device at the date of release
    Difference, //Used when detection method is not Exact or None. This is an integer value and the larger the value the less confident the detector is in this result.
    Drift, // Total difference in character positions where the substrings hashes were found away from where they were expected.
    UserAgents, // The matched User-Agents.

    // For unspecified fields
    Custom(&'static str),
}

impl PropertyName {
    pub fn to_str(&self) -> &str {
        match self {
            PropertyName::Custom(s) => s,
            _ => self.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Display, AsRefStr)]
pub enum EvidenceName {
    #[strum(serialize = "user-agent")]
    UserAgent,
    #[strum(serialize = "sec-ch-ua")]
    SecChUa,
    #[strum(serialize = "sec-ch-platform")]
    SecChPlatform,

    // For unspecified fields
    Custom(&'static str),
}

impl EvidenceName {
    pub fn value(self, v: &str) -> (EvidenceName, &str) {
        (self, v)
    }

    pub fn as_str(&self) -> &str {
        match self {
            EvidenceName::Custom(s) => s,
            _ => self.as_ref(),
        }
    }
}

type ResourceManager = Box<bindings::fiftyoneDegreesResourceManager>;
type Properties = bindings::fiftyoneDegreesPropertiesRequired;
type ConfigHash = bindings::fiftyoneDegreesConfigHash;

pub struct ManagerConfig {
    pub data_file_path: &'static Path,
    pub property_names: Option<&'static [PropertyName]>,
}

pub struct Evidence {
    evidence_ptr: *mut bindings::fiftyoneDegreesEvidenceKeyValuePairArray,
    evidence_data: Vec<(CString, CString)>,
}

impl Drop for Evidence {
    fn drop(&mut self) {
        unsafe {
            bindings::fiftyoneDegreesEvidenceFree(self.evidence_ptr);
        }
    }
}

impl Evidence {
    fn new(capacity: u32) -> FiftyOneDegreesResult<Self> {
        let evidence_ptr = unsafe { bindings::fiftyoneDegreesEvidenceCreate(capacity) };
        if evidence_ptr.is_null() {
            return Err(UnsafeOperationError(String::from(
                "Failed to create evidence object: got null",
            )));
        }
        Ok(Self {
            evidence_ptr,
            evidence_data: Vec::new(),
        })
    }

    fn add(&mut self, key: &str, val: &str) -> FiftyOneDegreesResult<()> {
        let key_cstring = build_cstring(CStringKind::EvidenceKey, key)?;
        let val_cstring = build_cstring(CStringKind::EvidenceValue, val)?;

        let added = unsafe {
            bindings::fiftyoneDegreesEvidenceAddString(
                self.evidence_ptr,
                bindings::e_fiftyone_degrees_evidence_prefix_FIFTYONE_DEGREES_EVIDENCE_HTTP_HEADER_STRING,
                key_cstring.as_ptr(),
                val_cstring.as_ptr(),
            )
        };
        if added.is_null() {
            return Err(UnsafeOperationError(format!(
                "Failed add evidence key={}: got null",
                key
            )));
        }

        self.evidence_data.push((key_cstring, val_cstring));
        Ok(())
    }
}

pub struct ResultData {
    results_ptr: *mut bindings::fiftyoneDegreesResultsHash,
}

impl Drop for ResultData {
    fn drop(&mut self) {
        unsafe {
            bindings::fiftyoneDegreesResultsHashFree(self.results_ptr);
        }
    }
}

impl ResultData {
    fn new(
        manager_ptr: *mut bindings::fiftyoneDegreesResourceManager,
        evidence_ptr: *mut bindings::fiftyoneDegreesEvidenceKeyValuePairArray,
    ) -> FiftyOneDegreesResult<Self> {
        let results_ptr = unsafe {
            bindings::fiftyoneDegreesResultsHashCreate(
                manager_ptr,
                // TODO: These values must be tuned according to passed evidence (for example we can do batch processing)
                1, // UA capacity
                0, // overrides disabled
            )
        };
        if results_ptr.is_null() {
            return Err(UnsafeOperationError(String::from(
                "Failed to create result object: got null",
            )));
        };
        let exception = null_mut();
        unsafe {
            bindings::fiftyoneDegreesResultsHashFromEvidence(results_ptr, evidence_ptr, exception)
        }
        verify_exception(exception, Operation::ApplyEvidence)?;
        Ok(Self { results_ptr })
    }

    pub fn get_value_as_string(
        &self,
        property_name: PropertyName,
    ) -> FiftyOneDegreesResult<Option<String>> {
        //let value = self.get_value(property_name)?;
        //Ok(value.map(|s| s.to_string()))
        let property_name_cstring =
            build_cstring(CStringKind::PropertyName, property_name.to_str())?;
        let mut buf = vec![0_i8; 64];
        let sep = build_cstring(CStringKind::HashResultSeparator, ", ")?;
        let exception = null_mut();

        let required_len = unsafe {
            bindings::fiftyoneDegreesResultsHashGetValuesString(
                self.results_ptr,
                property_name_cstring.as_ptr(),
                buf.as_mut_ptr(),
                buf.len(),
                sep.as_ptr(),
                exception,
            )
        };

        verify_exception(exception, Operation::ReadProperty)?;

        if required_len > buf.len() {
            return Err(UnsafeOperationError(format!(
                "Buffer too small for property: {}, expected: {}, actual: {}",
                property_name,
                required_len,
                buf.len()
            )));
        }

        if buf.len() == 0 {
            return Err(UnsafeOperationError(format!(
                "No data written for property: {}",
                property_name
            )));
        }

        let val_str = unsafe { CStr::from_ptr(buf.as_ptr()) }
            .to_string_lossy()
            .to_string();

        Ok(Some(val_str).filter(|s| !s.is_empty() && s != "Unknown" && s != "N/A"))
    }

    pub fn get_value(&self, property_name: &str) -> FiftyOneDegreesResult<Option<Cow<'_, str>>> {
        let property_name_cstring = build_cstring(CStringKind::PropertyName, property_name)?;
        let mut buf = vec![0_i8; 128];
        let sep = build_cstring(CStringKind::HashResultSeparator, ", ")?;
        let exception = null_mut();

        let required_len = unsafe {
            bindings::fiftyoneDegreesResultsHashGetValuesString(
                self.results_ptr,
                property_name_cstring.as_ptr(),
                buf.as_mut_ptr(),
                buf.len(),
                sep.as_ptr(),
                exception,
            )
        };

        verify_exception(exception, Operation::ReadProperty)?;

        if required_len > buf.len() {
            return Err(UnsafeOperationError(format!(
                "Buffer too small for property: {}, expected: {}, actual: {}",
                property_name,
                required_len,
                buf.len()
            )));
        }

        if buf.len() == 0 {
            return Err(UnsafeOperationError(format!(
                "No data written for property: {}",
                property_name
            )));
        }

        let val_str = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_string_lossy();

        Ok(Some(val_str).filter(|s| !s.is_empty()))
    }
}

pub struct Manager {
    instance: ResourceManager,
}

impl Drop for Manager {
    fn drop(&mut self) {
        unsafe {
            bindings::fiftyoneDegreesResourceManagerFree(self.instance.as_mut());
        }
    }
}

impl Manager {
    fn build_config() -> FiftyOneDegreesResult<ConfigHash> {
        //let mut config = Box::new(unsafe { bindings::fiftyoneDegreesHashHighPerformanceConfig });
        /*
        config.nodes.concurrency = 4;
        config.profiles.concurrency = 4;
        config.profileOffsets.concurrency = 4;
        config.rootNodes.concurrency = 4;
        config.values.concurrency = 4;
        config.strings.concurrency = 4;
        config.b.b.usesUpperPrefixedHeaders = false;
        config.b.updateMatchedUserAgent = false;
        */

        let config = unsafe { bindings::fiftyoneDegreesHashHighPerformanceConfig };
        Ok(config)
    }

    pub fn new(config: ManagerConfig) -> FiftyOneDegreesResult<Self> {
        verify_data_file_path(config.data_file_path)?;

        let path_cstring = config
            .data_file_path
            .canonicalize()
            .map_err(|e| IOError("Failed to canonicalize data file path", Some(e)))?
            .to_str()
            .ok_or_else(|| IOError("Failed to convert data file path to string", None))
            .and_then(|s| build_cstring(CStringKind::FilePath, s))?;

        let properties_cstring: CString; // This variable must survive until Manager is created (compiler doesn't check its lifecycle)
        let properties: *mut bindings::fiftyone_degrees_properties_required_t;
        if config.property_names.is_some() {
            let property_names = config
                .property_names
                .unwrap()
                .iter()
                .map(PropertyName::to_str)
                .join(",");

            properties_cstring = build_cstring(CStringKind::PropertyName, &property_names)?;

            let mut p = Properties {
                existing: null_mut(),
                array: null_mut(),
                string: properties_cstring.as_ptr(),
                count: 0,
            };
            properties = &mut p;
        } else {
            // All properties if not specified
            properties = null_mut();
        }

        let mut config = Self::build_config()?;
        //let mut manager = std::mem::MaybeUninit::<bindings::fiftyoneDegreesResourceManager>::uninit();
        let mut manager =
            Box::new(unsafe { std::mem::zeroed::<bindings::fiftyoneDegreesResourceManager>() });
        let exception = null_mut();

        let status = unsafe {
            bindings::fiftyoneDegreesHashInitManagerFromFile(
                manager.as_mut(),
                &mut config,
                properties,
                path_cstring.as_ptr(),
                exception,
            )
        };

        verify_exception(exception, Operation::InitManager)?;

        if status != bindings::e_fiftyone_degrees_status_code_FIFTYONE_DEGREES_STATUS_SUCCESS {
            return Err(InternalApiError(
                Operation::InitManager,
                status,
                status_to_error_message(status),
                "Status check failed",
            ));
        }

        Ok(Self { instance: manager })
    }

    /// Detects device properties based on the provided evidence.
    ///
    /// # Parameters
    /// - `evidence_data`: A slice of key-value pairs representing HTTP headers or client hints.
    ///
    /// # Returns
    /// - `Ok(ResultData)` containing device detection results.
    /// - `Err(FiftyOneDegreesError)` if detection fails.
    ///
    /// # Safety and Threading
    /// ⚠️ **Not thread-safe.**
    ///
    /// This method uses internal mutable state via FFI and must not be called concurrently
    /// from multiple threads or asynchronous tasks unless external synchronization is used.
    ///
    /// If thread-safe behavior is needed, consider using a `Mutex<Manager>` or other
    /// synchronization primitives to guard access to this function.
    ///
    /// # Example
    /// ```
    /// let result = manager.detect(&[("user-agent", "...")])?;
    /// ```
    // TODO: Update example, API has changed
    pub fn detect(
        &self,
        evidence_data: &[(EvidenceName, &str)],
    ) -> FiftyOneDegreesResult<ResultData> {
        if evidence_data.len() == 0 {
            return Err(AssertionError(
                Operation::CreateEvidence,
                "Evidence data must contain at least one item",
            ));
        }

        let mut evidence = Evidence::new(evidence_data.len() as u32)?;

        for (key, val) in evidence_data {
            evidence.add(key.as_str(), val)?;
        }

        let manager_ptr = self.instance.as_ref() as *const _ as *mut _;
        let result = ResultData::new(manager_ptr, evidence.evidence_ptr)?;
        Ok(result)
    }
}
