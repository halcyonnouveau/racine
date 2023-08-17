use crate::options::{Cli, Config, Record as CRecord};
use crate::util;
use maxminddb::{geoip2, Reader};
use rayon::prelude::*;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use tracing::error;
use trust_dns_resolver::Name;
use trust_dns_server::{
    authority::MessageResponseBuilder,
    client::rr::{rdata::MX, rdata::TXT, LowerName, RData, Record, RecordType},
    proto::op::{Header, MessageType, OpCode, ResponseCode},
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid OpCode {0:}")]
    InvalidOpCode(OpCode),
    #[error("Invalid MessageType {0:}")]
    InvalidMessageType(MessageType),
    #[error("IO error: {0:}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct Handler {
    records: Vec<CRecord>,
    reader: Reader<Vec<u8>>,
    use_ip: Option<String>,
}

impl Handler {
    /// Create new handler
    pub fn new(cli: &Cli, options: &Config) -> Self {
        let reader = maxminddb::Reader::open_readfile(&cli.mmdb)
            .expect("Could not open MaxMind DB database file");

        Handler {
            reader,
            records: options.records.clone(),
            use_ip: cli.use_ip.clone(),
        }
    }

    fn generate_record(
        &self,
        name: &LowerName,
        record_type: RecordType,
        value: &str,
        ttl: u32,
        preference: Option<u16>,
    ) -> Record {
        let rdata: Option<RData> = match record_type {
            RecordType::A => Some(RData::A(Ipv4Addr::from_str(value).unwrap())),
            RecordType::AAAA => Some(RData::AAAA(Ipv6Addr::from_str(value).unwrap())),
            RecordType::CAA => Some(RData::CAA(util::caa_from_string(value).unwrap())),
            RecordType::CNAME => Some(RData::CNAME(Name::from_str_relaxed(value).unwrap())),
            RecordType::MX => Some(RData::MX(MX::new(
                preference.expect("Should have preference with MX record"),
                Name::from_str_relaxed(value).unwrap(),
            ))),
            RecordType::NS => Some(RData::NS(Name::from_str_relaxed(value).unwrap())),
            RecordType::SOA => Some(RData::SOA(
                util::soa_from_string(value).expect("Should be a valid SOA record"),
            )),
            RecordType::SRV => Some(RData::SRV(
                util::srv_from_string(value).expect("Should be a valid SRV record"),
            )),
            RecordType::TXT => Some(RData::TXT(TXT::new(vec![value.to_string()]))),
            _ => None,
        };

        Record::from_rdata(name.into(), ttl, rdata.expect("Invalid record type"))
    }

    /// Handle request, returning ResponseInfo if response was successfully sent, or an error.
    async fn do_handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response: R,
    ) -> Result<ResponseInfo, Error> {
        // make sure the request is a query
        if request.op_code() != OpCode::Query {
            return Err(Error::InvalidOpCode(request.op_code()));
        }

        // make sure the message type is a query
        if request.message_type() != MessageType::Query {
            return Err(Error::InvalidMessageType(request.message_type()));
        }

        let request_ip = match &self.use_ip {
            Some(ip) => FromStr::from_str(&ip).unwrap(),
            None => request.src().ip(),
        };

        let request_name = request.query().name();
        let country: geoip2::City = self.reader.lookup(request_ip).unwrap();
        let continent_code = country.continent.and_then(|c| c.code).unwrap_or("");
        let country_code = country.country.and_then(|c| c.iso_code).unwrap_or("");

        // create zone records
        let records: Vec<Record> = self
            .records
            .par_iter()
            .flat_map(|record| {
                let zone_name = LowerName::from_str(&record.name).expect("Invalid record name");
                if zone_name.zone_of(request_name) {
                    let records: Vec<Option<Record>> = record
                        .geo
                        .par_iter()
                        .map(|geo| {
                            let ttl = match geo.ttl {
                                Some(ttl) => ttl,
                                None => record.ttl,
                            };

                            match (&geo.country, &geo.continent) {
                                (Some(c), _) if *c == country_code => Some(self.generate_record(
                                    request_name,
                                    record.record_type,
                                    &geo.value,
                                    ttl,
                                    record.preference,
                                )),
                                (_, Some(b)) if *b == continent_code => Some(self.generate_record(
                                    request_name,
                                    record.record_type,
                                    &geo.value,
                                    ttl,
                                    record.preference,
                                )),
                                _ => None,
                            }
                        })
                        .collect();

                    let unwrapped_records: Vec<Record> = records
                        .into_iter()
                        .filter_map(|opt_value| opt_value)
                        .collect();

                    if unwrapped_records.is_empty() {
                        vec![self.generate_record(
                            request_name,
                            record.record_type,
                            &record.value,
                            record.ttl,
                            record.preference,
                        )]
                    } else {
                        unwrapped_records
                    }
                } else {
                    Vec::new()
                }
            })
            .collect();

        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);
        let res = builder.build(header, records.iter(), &[], &[], &[]);
        Ok(response.send_response(res).await?)
    }
}

#[async_trait::async_trait]
impl RequestHandler for Handler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response: R,
    ) -> ResponseInfo {
        match self.do_handle_request(request, response).await {
            Ok(info) => info,
            Err(error) => {
                error!("Error in RequestHandler: {error}");
                let mut header = Header::new();
                header.set_response_code(ResponseCode::ServFail);
                header.into()
            }
        }
    }
}
