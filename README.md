# dw ⚡
A simple and fast command-line tool for downloading files.

dw is a download accelerator that gets you the files you need, faster. It's designed to be a lightweight and powerful tool that speeds up downloads by using a smart, multi-connection approach. It's perfect for anyone who regularly downloads large files and wants to save time.

While it works hard under the hood to maximize your download speed, it automatically handles all the details for you. All you have to do is give it a link.

## How to Get It
If you have the Rust programming language installed, you can get `dw` with a single command:

```bash
cargo install dw
```

ou can also build the tool from its source code.

## How to Use it
Using `dw` is as easy as it gets. To download a file, just provide the link:

```bash
dw "http://cachefly.cachefly.net/100mb.test"
```

The tool shows a clean proress bar so you can see your download's speed and how much time is left.

If you want to save the file with a different name, use the `-o` flag:

```bash
dw "http://cachefly.cachefly.net/100mb.test" -o my_file.zip
```

For even faster speeds, you can increase the number of connections with the `-c` flag. The default is 8, but you can set it higher or lower depending on your network.

```bash
dw "http://cachefly.cachefly.net/100mb.test" -c 16
```
