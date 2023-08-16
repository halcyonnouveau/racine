# racine

A basic DNS server with geo-lookup for multi-region routing.

## Getting Started

> **NOTE:** `racine` requires the GeoLite2 Country database from MaxMind. For more information and to sign up for the download, [go here](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data).

### Installing

Download the latest binary from the [releases page](https://github.com/halcyonnouveau/racine/releases).

Or install it with `cargo`.

```bash
cargo install racine
```

### Usage

Create a YAML configuration file with your records:

```yaml
records:
  # basic example
  - name: racine.fun # domain name
    type: A          # DNS record type
    value: 127.0.0.1 # value of record
    ttl: 30          # ttl (optional) defaults to 86400
  # example with geolocation
  - name: racine.fun
    type: CNAME
    value: au.racine.fun. # default value
    geo:
      - country: NZ # ISO country code
        value: nz.racine.fun.
      - continent: EU
        value: eu.racine.fun.
```

Run `racine` with:

```bash
racine --config /path/to/config.yaml --maxmind /path/to/geolite.mmdb
```
