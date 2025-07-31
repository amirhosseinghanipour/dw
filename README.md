# dw ⚡

`dw` is a blazingly fast, parallel download accelerator written in Rust.

---

## 🚀 Installation

Getting `dw` is easy. You can choose any of the following methods.

#### From Crates.io (Recommended)

If you have the Rust toolchain installed, you can install `dw` directly from the official package registry:

```sh
cargo install dw-rs
```

#### From Source

If you prefer to build it yourself, you can compile `dw` directly from the source code:

```sh
git clone https://github.com/amirhosseinghanipour/dw.git
cd dw
cargo build --release
```

The executable will be in `./target/release/dw`.

---

## 🏁 Getting Started

Using `dw` is straightforward. Here are the most common commands to get you started.

#### Basic Download

The simplest way to use `dw` is to just give it a URL. The tool will automatically figure out the filename and show you a progress bar while it downloads.

```sh
dw "http://cachefly.cachefly.net/100mb.test"
```

#### Saving with a Different Name

If you want to save the file with a different name, use the `-o` (or `--output`) flag.

```sh
dw "http://cachefly.cachefly.net/100mb.test" -o "100mb.zip"
```

---

## 🛠️ Advanced Usage

For those who want more control over the download process, `dw` offers several flags to fine-tune its behavior.

#### Change the Number of Connections

By default, `dw` uses 8 connections to download a file. You can increase or decrease this number with the `-c` flag to find the sweet spot for your network.

```sh
dw "http://cachefly.cachefly.net/100mb.test" -c 16
```

#### Adjust the Buffer Size

You can change the size of the memory buffer used for writing data to disk with the `-b` flag. The value is in kilobytes.

```sh
dw "http://cachefly.cachefly.net/100mb.test" -b 2048
```

#### Enable Adaptive Buffering

For a smarter download process, you can enable adaptive buffering with the `--adaptive` flag. When turned on, `dw` will automatically adjust its buffer size based on your real-time download speed.

```sh
dw "http://cachefly.cachefly.net/100mb.test" --adaptive
```

#### Set the Minimum File Size for Parallelism

`dw` is smart enough not to use its parallel download feature for very small files where it wouldn't be efficient. You can control this threshold with the `--min-chunk` flag, which sets the minimum file size (in megabytes) required to trigger a parallel download.

```sh
dw "[http://cachefly.cachefly.net/100mb.test](http://cachefly.cachefly.net/100mb.test)" --min-chunk 10
```

---

## License

This project is open-source and available under the [**GPL-3.0 License**](LICENSE).
