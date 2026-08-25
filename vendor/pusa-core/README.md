# vendor/pusa-core

闭源 `pusa-core` 源码打出的 **`libpusa_core` cdylib**，按 rustc target triple 分目录，**随开源仓一起发布**。

```sh
just pusa-core
# 或：cd pusa-core && just pack
```

`just desktop` 会先跑 `ensure-pusa-core`：当前 triple 没有产物，或 `pusa-core`/`protocol` 源码比产物新时，自动重新打包。

运行时可设 `PUSA_CORE_LIB=/abs/path/to/libpusa_core.dylib`。
