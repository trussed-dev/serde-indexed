# Changelog

## [Unreleased][]

[Unreleased]: https://github.com/trussed-dev/serde-indexed/compare/0.2.0...HEAD

-

## [v0.2.0][] (2025-06-05)

[v0.2.0]: https://github.com/trussed-dev/serde-indexed/compare/0.1.1...0.2.0

- Add support for `#[serde(with)]` ([#16][])
- Add support for `#[serde(skip)]` ([#14][])
- Add support for generics ([#11][])
- skip_serializing_if no longer incorrectly affects deserialization (fixes [#2][])
- No longer fails deserialising maps with unknown fields ([#19][])
- Prefer explicit indexing over automatically assigned indices ([#17][]):
  - Require `auto_index` attribute to enable automatic index assignment
  - Add `index` attribute for explicit index assignment

[#2]: https://github.com/trussed-dev/serde-indexed/issues/2
[#11]: https://github.com/trussed-dev/serde-indexed/pull/11
[#14]: https://github.com/trussed-dev/serde-indexed/pull/14
[#16]: https://github.com/trussed-dev/serde-indexed/pull/16
[#17]: https://github.com/trussed-dev/serde-indexed/issues/17
[#19]: https://github.com/trussed-dev/serde-indexed/pull/19

## [v0.1.1][] (2024-04-03)

[v0.1.1]: https://github.com/trussed-dev/serde-indexed/compare/0.1.0...0.1.1

- Add support for lifetime generics ([#6][])
- Migrate to syn2 ([#7][])
- Migrate to edition 2021 ([#8][])

[#6]: https://github.com/trussed-dev/serde-indexed/pull/6
[#7]: https://github.com/trussed-dev/serde-indexed/pull/7
[#8]: https://github.com/trussed-dev/serde-indexed/pull/8

## [v0.1.0][] (2021-02-01)

[v0.1.0]: https://github.com/trussed-dev/serde-indexed/releases/tag/0.1.0

Initial release.
