# Build

## 必要依赖
* [Rust (Nightly)](https://www.rust-lang.org)

## 编译(Release)
`cargo +nightly build --release`

### 开启features
使用`--features`标志开启指定feature，使用逗号分割.

`cargo +nightly build --release --features=featureA,featureB`

使用`--no-default-features`标志禁用默认的feature.

`cargo +nightly build --release --no-default-features`
