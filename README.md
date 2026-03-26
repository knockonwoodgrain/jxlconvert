# JXLCONVERT - Convert your image files to JPEG XL while preserving metadata!

jxlconvert uses `libjxl` or `libvips` as the encoders & `exiftool` to write metadata.
You will need to install all three programs and have them on your path for jxlconvert to work.

You can install them here:

1. [libvips: an image processing library](https://github.com/libvips/libvips) \
2. [libjxl: JPEG XL reference implementation](https://github.com/libjxl/libjxl)\
3. [ExifTool by Phil Harvey](https://exiftool.org/)

> [!NOTE]
> You'll also need the Brotli Compress Algoritm Perl Library, that exiftool uses to read brotli compressed metadata
> You can install it on Arch using 
> ```shell
> pacman -Sy brotli perl-io-compress-brotli
> ``` 
> it's literally that easy, this is specifically for Swarit
> If you're on Windows, it's complicated as shit for some reason, so I plan to use the libjxl encoder metadata preserver as an option soon,
> But until then you'll need to install [Strawberry Perl](https://strawberryperl.com/) and then install it using `cpan IO::Compress::Brotli` in the perl shell. 

Here's what the `--help` command outputs: 
```shell
JXLCONVERT Batch convert image files to JXL

Usage: jxlconvert [OPTIONS] [PATH] [OUTPUT_PATH]

Arguments:
  [PATH]         [default: .]
  [OUTPUT_PATH]  [default: ./JXL]

Options:
  -e, --encoder <cjxl> <vips>  [default: cjxl]
  -q, --quality <0..100>       [default: 85]
  -h, --help                   Print help
  -V, --version                Print version
```



# Why should I use JPEG XL 
Because JPEG XL is fucking awesome, read more on [JPEGXL.INFO](https://jpegxl.info/)\
Battle of the Codecs [Comparision Chart](https://jpegxl.info/resources/battle-of-codecs.html)\
Can I use it? (You can't use it yet, but it's the next JPEG, and even Chrome has turned around and started implementation) [JPEG XL](https://caniuse.com/jpegxl)
