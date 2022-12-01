# zcat-rs

ZipCat, cat zip file from http url.

## Usage

```
Usage: zcat [OPTIONS] <ZIP_FILE> [FILE_LIST]...

Arguments:
  <ZIP_FILE>      Read from this file
  [FILE_LIST]...  Read those files

Options:
  -l, --list
  -h, --help     Print help information
  -V, --version  Print version information
```

## Examples

1. list file

```shell
$ ./target/debug/zcat -l http://arms-apm-cn-hangzhou.oss-cn-hangzhou.aliyuncs.com/ms/AliyunJavaAgent.zip
 Length       Date    Time    Name
---------  ---------- -----   ----
        0  2022-11-15 10:17   AliyunJavaAgent/
       29  2022-11-15 10:17   AliyunJavaAgent/version
......
---------                     -------
 91499485                     283 files
```

2. cat file from zip file url

```shell
$ ./target/debug/zcat http://arms-apm-cn-hangzhou.oss-cn-hangzhou.aliyuncs.com/ms/AliyunJavaAgent.zip AliyunJavaAgent/version
20221115101704_55a55f5_2.8.0
```
