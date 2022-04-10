![crates.io](https://img.shields.io/crates/v/polyline-codec?style=flat-square) ![docs.rs](https://img.shields.io/docsrs/polyline-codec?style=flat-square)
![Crates.io](https://img.shields.io/crates/l/polyline-codec?style=flat-square)

# Rust port of Google Maps Polyline Encoding

## Description

Encode and decode polyines in Rust using this package.

Polyline encoding is a lossy compression algorithm that allows you to store a series of coordinates as a single string. Point coordinates are encoded using signed values.

Read more at https://developers.google.com/maps/documentation/utilities/polylinealgorithm.

## Note

I have no affiliation with Google or Google Maps. This package was ported from https://github.com/googlemaps/js-polyline-codec.

## Example

```rust
const encoded = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
println!(polyline_codec::decode(encoded, 5).unwrap());
// [
//    LatLng(
//        38.5,
//        -120.2,
//    ),
//    LatLng(
//        40.7,
//        -120.95,
//    ),
//    LatLng(
//        43.252,
//        -126.453,
//    ),
// ]

const path = [
  (38.5, -120.2),
  (40.7, -120.95),
  (43.252, -126.453),
];
println!(polyline_codec::encode(path, 5).unwrap());
// "_p~iF~ps|U_ulLnnqC_mqNvxq`@"
```

## License

MIT OR Apache v2.0
