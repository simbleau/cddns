Changelog
=========
This changelog follows the patterns described here: https://keepachangelog.com/en/1.0.0/.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

## Unreleased
### added
- All configuration options are now built with `config build`
- Added verbose logging with `-v`
- Added warning for empty inventory
### changed
- Improved readability of errors with `inventory watch`
- Improved readability of `config show`
- `cddns list zones -z <name|id>` now only matches one zone result
- `cddns list records -z <name|id>` now only matches one zone result
- `cddns list records -r <name|id>` now only matches one record result
### deprecated
- Placeholder
### removed
- Placeholder
### fixed
- Environment variables work for all commands
### security
- Placeholder

## 0.1.2
### security
- Updated clap: 4.0.18 -> 4.0.23
- Updated regex: 1.6.0 -> 1.7.0

## 0.1.1
### changed
- Configuration path no longer needs to exist

## 0.1.0
- Initialize release.