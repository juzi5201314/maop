# Build

## 必要依赖
* [Rust (Nightly)](https://www.rust-lang.org)

## 编译(Release)
```cargo +nightly build --release```

### 开启features
使用`--features`标志开启指定feature，使用逗号分割.

```
cargo +nightly build --release --features=featureA,core/featureB
```

使用`--no-default-features`标志禁用默认的feature.

```
cargo +nightly build --release --no-default-features
```

## Features
* 默认: `snmalloc`
* `core/snmalloc`: 使用[snmalloc](https://github.com/microsoft/snmalloc)分配器.
* `core/native-cpu`: 开启snmalloc/native-cpu feature.
* `http/session_store_rocksdb`: 使用[rocksdb](https://github.com/facebook/rocksdb)储存session而不是默认的文件储存.

## 优化
release编译默认已经进行了lto与strip，
但可以使用`target-cpu=native`来启用潜在的针对本机cpu的优化:

```
RUSTFLAGS="-C target-cpu=native" cargo ...
```
