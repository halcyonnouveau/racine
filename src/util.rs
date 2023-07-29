use trust_dns_resolver::Name;
use trust_dns_server::client::rr::{rdata::caa::Property, rdata::CAA, rdata::SOA, rdata::SRV};
use url::Url;

/// Example:
/// 0 issue "ca.example.net"
pub fn caa_from_string(input: &str) -> Option<CAA> {
    let mut parts = input.split_whitespace();
    let issuer_critical = parts.next()?.to_string() == "1";
    let tag = Property::from(parts.next()?.to_string());
    let value = parts.next()?;

    match tag {
        Property::Issue => Some(CAA::new_issue(
            issuer_critical,
            Some(Name::from_str_relaxed(value.to_string()).unwrap()),
            Vec::new(),
        )),
        Property::IssueWild => Some(CAA::new_issuewild(
            issuer_critical,
            Some(Name::from_str_relaxed(value.to_string()).unwrap()),
            Vec::new(),
        )),
        Property::Iodef => Some(CAA::new_iodef(issuer_critical, Url::parse(value).unwrap())),
        _ => None,
    }
}

pub fn soa_from_string(input: &str) -> Option<SOA> {
    let mut parts = input.split_whitespace();
    let mname = Name::from_str_relaxed(parts.next()?.to_string()).unwrap();
    let rname = Name::from_str_relaxed(parts.next()?.to_string()).unwrap();
    let serial = parts.next()?.parse().ok()?;
    let refresh = parts.next()?.parse().ok()?;
    let retry = parts.next()?.parse().ok()?;
    let expire = parts.next()?.parse().ok()?;
    let minimum = parts.next()?.parse().ok()?;

    Some(SOA::new(
        mname, rname, serial, refresh, retry, expire, minimum,
    ))
}

pub fn srv_from_string(input: &str) -> Option<SRV> {
    let mut parts = input.split_whitespace();
    let priority = parts.next()?.parse().ok()?;
    let weight = parts.next()?.parse().ok()?;
    let port = parts.next()?.parse().ok()?;
    let target = Name::from_str_relaxed(parts.next()?.to_string()).unwrap();

    Some(SRV::new(priority, weight, port, target))
}
