use crate::models::Domain;
use ddns_core::WebDomain;

impl From<Domain> for WebDomain {
    fn from(domain: Domain) -> Self {
        WebDomain { hostname: domain.hostname, current_ip: domain.current_ip }
    }
}
