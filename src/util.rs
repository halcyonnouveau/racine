use trust_dns_resolver::Name;
use trust_dns_server::client::rr::{rdata::SOA, rdata::SRV};

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
