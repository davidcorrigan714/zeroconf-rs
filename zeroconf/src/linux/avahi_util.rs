//! Utilities related to Avahi

use crate::{NetworkInterface, ServiceType};
use avahi_sys::{avahi_address_snprint, avahi_strerror, AvahiAddress};
use libc::c_char;
use std::ffi::CStr;

/// Converts the specified `*const AvahiAddress` to a `String`.
///
/// The new `String` is constructed through allocating a new `CString`, passing it to
/// `avahi_address_snprint` and then converting it to a Rust-type `String`.
///
/// # Safety
/// This function is unsafe because of internal Avahi calls and raw pointer dereference.
pub unsafe fn avahi_address_to_string(addr: *const AvahiAddress) -> String {
    assert_not_null!(addr);

    let addr_str = c_string!(alloc(avahi_sys::AVAHI_ADDRESS_STR_MAX as usize));

    avahi_address_snprint(
        addr_str.as_ptr() as *mut c_char,
        avahi_sys::AVAHI_ADDRESS_STR_MAX as usize,
        addr,
    );

    String::from(addr_str.to_str().unwrap())
        .trim_matches(char::from(0))
        .to_string()
}

/// Returns the `&str` message associated with the specified error code.
pub fn get_error<'a>(code: i32) -> &'a str {
    unsafe {
        CStr::from_ptr(avahi_strerror(code))
            .to_str()
            .expect("could not fetch Avahi error string")
    }
}

/// Converts the specified [`NetworkInterface`] to the Avahi expected value.
///
/// [`NetworkInterface`]: ../../enum.NetworkInterface.html
pub fn interface_index(interface: NetworkInterface) -> i32 {
    match interface {
        NetworkInterface::Unspec => avahi_sys::AVAHI_IF_UNSPEC,
        NetworkInterface::AtIndex(i) => i as i32,
    }
}

/// Executes the specified closure and returns a formatted `Result`
pub fn sys_exec<F: FnOnce() -> i32>(func: F, message: &str) -> crate::Result<()> {
    let err = func();

    if err < 0 {
        crate::Result::Err(
            format!(
                "{}: `{}`",
                message,
                crate::linux::avahi_util::get_error(err)
            )
            .into(),
        )
    } else {
        crate::Result::Ok(())
    }
}

/// Formats the specified `ServiceType` as a `String` for use with Avahi
pub fn format_service_type(service_type: &ServiceType) -> String {
    format!("_{}._{}", service_type.name(), service_type.protocol())
}

/// Formats the specified `ServiceType` as a `String` for browsing Avahi services
pub fn format_browser_type(service_type: &ServiceType) -> String {
    let kind = format_service_type(service_type);
    let sub_types = service_type.sub_types();

    if sub_types.len() == 0 {
        return kind;
    }

    if sub_types.len() > 1 {
        warn!("browsing by multiple sub-types is not supported on Avahi devices, using first sub-type only");
    }

    format_sub_type(&sub_types[0], &kind)
}

/// Formats the specified `sub_type` string as a `String` for use with Avahi
pub fn format_sub_type(sub_type: &str, kind: &str) -> String {
    format!(
        "{}{}._sub.{}",
        if sub_type.starts_with('_') { "" } else { "_" },
        sub_type,
        kind
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use avahi_sys::{
        AvahiAddress__bindgen_ty_1, AvahiIPv4Address, AvahiIPv6Address, AVAHI_PROTO_INET,
        AVAHI_PROTO_INET6,
    };

    #[test]
    fn format_service_type_returns_valid_string() {
        assert_eq!(
            format_service_type(&ServiceType::new("http", "tcp").unwrap()),
            "_http._tcp"
        );
    }

    #[test]
    fn format_sub_type_returns_valid_string() {
        assert_eq!(format_sub_type("foo", "_http._tcp"), "_foo._sub._http._tcp");
    }

    #[test]
    fn format_sub_type_strips_leading_underscore() {
        assert_eq!(
            format_sub_type("_foo", "_http._tcp"),
            "_foo._sub._http._tcp"
        );
    }

    #[test]
    fn get_error_returns_valid_error_string() {
        assert_eq!(get_error(avahi_sys::AVAHI_ERR_FAILURE), "Operation failed");
    }

    #[test]
    fn address_to_string_returns_correct_ipv4_string() {
        let ipv4_addr = AvahiAddress {
            proto: AVAHI_PROTO_INET,
            data: AvahiAddress__bindgen_ty_1 {
                ipv4: AvahiIPv4Address {
                    address: 0x6464a8c0, // 192.168.100.100
                },
            },
        };

        unsafe {
            assert_eq!(avahi_address_to_string(&ipv4_addr), "192.168.100.100");
        }
    }

    #[test]
    fn address_to_string_returns_correct_ipv6_string() {
        let ipv6_addr = AvahiAddress {
            proto: AVAHI_PROTO_INET6,
            data: AvahiAddress__bindgen_ty_1 {
                ipv6: AvahiIPv6Address {
                    address: [
                        0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78,
                        0x9a, 0xbc, 0xde, 0xf0,
                    ],
                },
            },
        };

        unsafe {
            assert_eq!(
                avahi_address_to_string(&ipv6_addr),
                "fe80::1234:5678:9abc:def0"
            );
        }
    }
}
