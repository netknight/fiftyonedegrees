use super::super::device_detection::{
    EvidenceName, Manager, ManagerConfig, PropertyName,
};

#[test]
fn test_device_detect() -> Result<(), Box<dyn std::error::Error>> {
    let data_file_path = std::path::Path::new("data.hash");
    //let data_file_path = std::path::Path::new("data_free2.hash");
    eprintln!("Data path: {:?}", data_file_path);

    let conf = ManagerConfig {
        data_file_path,
        property_names: Some(&[
            PropertyName::BrowserName,
            PropertyName::DeviceType,
            PropertyName::PlatformName,
            PropertyName::PlatformVersion,
            PropertyName::IsMobile,
        ]),
    };

    let manager = Manager::new(conf)?;

    let evidence_data = &[
        EvidenceName::UserAgent.value("Mozilla/5.0 (iPhone; CPU iPhone OS 15_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.2 Mobile/15E148 Safari/604.1"),
        EvidenceName::SecChUa.value("\"Safari\";v=\"15\", \"Mobile Safari\";v=\"15\", \"Chromium\";v=\"110\""),
        EvidenceName::SecChPlatform.value("\"iOS\""),
    ];

    let res = manager.detect(evidence_data)?;

    let browser_name = res.get_value_as_string(PropertyName::BrowserName)?;
    let device_type = res.get_value_as_string(PropertyName::DeviceType)?;
    let platform_name = res.get_value_as_string(PropertyName::PlatformName)?;
    let platform_version = res.get_value_as_string(PropertyName::PlatformVersion)?;
    let is_mobile = res.get_value_as_string(PropertyName::IsMobile)?;
    let unknown = res.get_value_as_string(PropertyName::Custom("Unknown"))?;

    assert_eq!(browser_name, Some(String::from("Mobile Safari")));
    assert_eq!(device_type, Some(String::from("SmartPhone")));
    assert_eq!(platform_name, Some(String::from("iOS")));
    assert_eq!(platform_version, Some(String::from("15.2")));
    assert_eq!(is_mobile, Some(String::from("True")));
    assert_eq!(unknown, None);

    Ok(())
}
