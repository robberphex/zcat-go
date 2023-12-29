# zcat-go

ZipCat, cat zip file from http url.

## Usage

```
zcat

Usage:
  zcat [flags] [file/url] file [...file]

Flags:
  -h, --help   help for zcat
  -l, --list   list files in zip
```

## Examples

1. list file

```shell
$ ./zcat -l \
  http://arms-apm-cn-hangzhou.oss-cn-hangzhou.aliyuncs.com/ms/AliyunJavaAgent.zip
   Length        Date  Time   Name
---------  ---------- -----   ----
        0  2023-09-30 17:32   AliyunJavaAgent/
       33  2023-09-30 17:32   AliyunJavaAgent/version
......
---------                     -------
 96300319                     302 files
```

2. cat file from zip file url

```shell
$ ./zcat \
  http://arms-apm-cn-hangzhou.oss-cn-hangzhou.aliyuncs.com/ms/AliyunJavaAgent.zip \
  AliyunJavaAgent/version
20230930173250_60ec150_2.9.2-mse
```
