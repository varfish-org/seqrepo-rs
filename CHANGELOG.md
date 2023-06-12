# Changelog

## [0.6.0](https://www.github.com/bihealth/seqrepo-rs/compare/v0.5.1...v0.6.0) (2023-06-12)


### Features

* more thread safety ([#62](https://www.github.com/bihealth/seqrepo-rs/issues/62)) ([3a6e72e](https://www.github.com/bihealth/seqrepo-rs/commit/3a6e72ea725cc42ceb6215a7dc615db742d1ce58))

### [0.5.1](https://www.github.com/bihealth/seqrepo-rs/compare/v0.5.0...v0.5.1) (2023-06-06)


### Bug Fixes

* depend on noodles-* rather noodles/features ([#60](https://www.github.com/bihealth/seqrepo-rs/issues/60)) ([#61](https://www.github.com/bihealth/seqrepo-rs/issues/61)) ([51406f2](https://www.github.com/bihealth/seqrepo-rs/commit/51406f2bfe1e6655a7e9eb390fea1feb9f27fb79))
* removing unused dependencies ([#57](https://www.github.com/bihealth/seqrepo-rs/issues/57)) ([756c2be](https://www.github.com/bihealth/seqrepo-rs/commit/756c2bec2430fd72f302740bd2308e451ef62f38))

## [0.5.0](https://www.github.com/bihealth/seqrepo-rs/compare/v0.4.0...v0.5.0) (2023-05-23)


### Features

* losening dependencies ([#55](https://www.github.com/bihealth/seqrepo-rs/issues/55)) ([e467af5](https://www.github.com/bihealth/seqrepo-rs/commit/e467af5bb46f89004bdde791618bfaf017fa4eee))

## [0.4.0](https://www.github.com/bihealth/seqrepo-rs/compare/v0.3.0...v0.4.0) (2023-04-24)


### Features

* switch from anyhow to thiserr ([#45](https://www.github.com/bihealth/seqrepo-rs/issues/45)) ([#46](https://www.github.com/bihealth/seqrepo-rs/issues/46)) ([497c17a](https://www.github.com/bihealth/seqrepo-rs/commit/497c17ae308fc2c2e18b67e4adf7277dfd4e13f1))

## [0.3.0](https://www.github.com/bihealth/seqrepo-rs/compare/v0.2.3...v0.3.0) (2023-03-31)


### Features

* moving cli to be an example ([#35](https://www.github.com/bihealth/seqrepo-rs/issues/35)) ([#36](https://www.github.com/bihealth/seqrepo-rs/issues/36)) ([0389c65](https://www.github.com/bihealth/seqrepo-rs/commit/0389c65bdeede3eb9b4ba459a227b9f075408644))

### [0.2.3](https://www.github.com/bihealth/seqrepo-rs/compare/v0.2.2...v0.2.3) (2023-03-14)


### Bug Fixes

* print SQL query in trace level only ([#32](https://www.github.com/bihealth/seqrepo-rs/issues/32)) ([#33](https://www.github.com/bihealth/seqrepo-rs/issues/33)) ([1aac31e](https://www.github.com/bihealth/seqrepo-rs/commit/1aac31e30d86cf6d5d77ce75b2cfbaba28410044))

### [0.2.2](https://www.github.com/bihealth/seqrepo-rs/compare/v0.2.1...v0.2.2) (2023-03-03)


### Bug Fixes

* open options to cache FASTA file ([#30](https://www.github.com/bihealth/seqrepo-rs/issues/30)) ([983e182](https://www.github.com/bihealth/seqrepo-rs/commit/983e182ec720e09b2e3abca13fe75dfe3b83aa79))

### [0.2.1](https://www.github.com/bihealth/seqrepo-rs/compare/v0.2.0...v0.2.1) (2023-02-24)


### Bug Fixes

* do not write out values twice when cache writing ([#24](https://www.github.com/bihealth/seqrepo-rs/issues/24)) ([#25](https://www.github.com/bihealth/seqrepo-rs/issues/25)) ([d497e53](https://www.github.com/bihealth/seqrepo-rs/commit/d497e53c19a3a1165fb11f53d0c4b6cd11da62b1))

## [0.2.0](https://www.github.com/bihealth/seqrepo-rs/compare/v0.1.2...v0.2.0) (2023-02-23)


### Features

* create cache writing and reading SeqRepo ([#21](https://www.github.com/bihealth/seqrepo-rs/issues/21)) ([#22](https://www.github.com/bihealth/seqrepo-rs/issues/22)) ([2d45533](https://www.github.com/bihealth/seqrepo-rs/commit/2d45533831183867b16ccbd934c7c953f418270a))

### [0.1.2](https://www.github.com/bihealth/seqrepo-rs/compare/v0.1.1...v0.1.2) (2023-02-16)


### Bug Fixes

* Cargo.toml for license-file ([#16](https://www.github.com/bihealth/seqrepo-rs/issues/16)) ([197c7e1](https://www.github.com/bihealth/seqrepo-rs/commit/197c7e1c48fd14d98fb73c9f796ff575b485441d))

### [0.1.1](https://www.github.com/bihealth/seqrepo-rs/compare/v0.1.0...v0.1.1) (2023-02-16)


### Bug Fixes

* release 0.1.1 to crates.io ([#14](https://www.github.com/bihealth/seqrepo-rs/issues/14)) ([9e3198b](https://www.github.com/bihealth/seqrepo-rs/commit/9e3198b55f47820d37b34d5560e1b5f6107badf9))

## 0.1.0 (2023-02-16)


### Features

* implement "seqrepo-cli export" for demo purposes ([#9](https://www.github.com/bihealth/seqrepo-rs/issues/9)) ([#10](https://www.github.com/bihealth/seqrepo-rs/issues/10)) ([b7301c7](https://www.github.com/bihealth/seqrepo-rs/commit/b7301c7bbb9ec1bd9b8a6b6d02b07a7e5b71820a))
* implement read-only access to aliases sqlite3 db ([#1](https://www.github.com/bihealth/seqrepo-rs/issues/1)) ([#2](https://www.github.com/bihealth/seqrepo-rs/issues/2)) ([26539eb](https://www.github.com/bihealth/seqrepo-rs/commit/26539ebfcd92f3465fc5e56e9011941c947c0514))
* implement SeqRepo based on FastaDir and AliasDb ([#7](https://www.github.com/bihealth/seqrepo-rs/issues/7)) ([#8](https://www.github.com/bihealth/seqrepo-rs/issues/8)) ([99922d6](https://www.github.com/bihealth/seqrepo-rs/commit/99922d6cd8c1dca711f7268de598e78417990829))
* port over (read-only) FastaDir ([#5](https://www.github.com/bihealth/seqrepo-rs/issues/5)) ([#6](https://www.github.com/bihealth/seqrepo-rs/issues/6)) ([d425221](https://www.github.com/bihealth/seqrepo-rs/commit/d42522183f2395c219ab75f24673a1b14436ff47))
