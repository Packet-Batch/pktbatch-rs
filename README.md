[Packet Batch](https://github.com/Packet-Batch) is a collection of high-performance tools used for generating network packets. These tools are commonly used for penetration testing, benchmarking, and network monitoring.

This repository serves as the Rust implementation of Packet Batch. While Packet Batch was originally written in C ([`pktbatch-c`](https://github.com/Packet-Batch/pktbatch-c)), this Rust version aims to provide a safer and more modern codebase while maintaining high performance.

That said, this will now be the main repository for Packet Batch. While `pktbatch-c` will still be maintained, all new features and improvements will be developed in this Rust implementation. The C version will only receive critical bug fixes.

## 🚨 Experimental & Incomplete
The Rust implementation of Packet Batch is currently in the early stages of development and is not yet complete. While a basic configuration works, many features may still be missing.

Please use [`pktbatch-c`](https://github.com/Packet-Batch/pktbatch-c) for now if you require a fully functional version. The Rust version is still a work in progress, but it will eventually reach feature parity with the C version and then some!

## 🚀 Features
* Fast packet generation using technologies such as [AF_XDP sockets](https://docs.kernel.org/networking/af_xdp.html).
    * ⚠️ AF_XDP support is currently the only available option, but support for other technologies (e.g., DPDK, AF_PACKET, etc.) will likely be added in the future!
* Highly configurable packet generation with support for various protocols and custom payloads.
    * Hostname and DNS lookup support.
    * Supports generating random source and destination IP addresses, ports, and other header fields like the IP TTL and ID fields.
* Support for executing multiple batches of packets with different configurations.
* Command-line interface that includes arguments for overriding the first batch's configuration without modifying the configuration file on disk.

## 🛠️ Building
### XDP Tools (LibXDP, LibBPF, etc.)
You will need to install [`xdp-tools`](https://github.com/xdp-project/xdp-tools) and its dependencies to build the AF_XDP version of the project (only version available at this time). I recommend following the instructions from [here](https://github.com/xdp-project/xdp-tutorial/blob/main/setup_dependencies.org).

Here are commands I run on Debian-based systems to set up the dependencies:

```bash
# Install dependencies
sudo apt update

sudo apt install -y clang llvm libelf-dev libpcap-dev build-essential libc6-dev-i386 m4 git

# Clone and build xdp-tools
git clone --recursive https://github.com/xdp-project/xdp-tools

cd xdp-tools

# Build and install xdp-tools
make

# Install xdp-tools to the system.
sudo make install

# While it shouldn't be technically needed, I always build and install LibBPF which is a sub-module inside of `xdp-tools`.
# Change to the libbpf directory.
cd lib/libbpf/src

# Build and install libbpf
make

sudo make install
```

### The Project
After installing the dependencies above, it's time to build the main project. Firstly, let's quickly clone the repository.

```bash
# Clone the repository
git clone https://github.com/Packet-Batch/pktbatch-rs

# Change to the project directory
cd pktbatch-rs
```

You **must** have Rust installed on your system. You can install Rust using [`rustup`](https://rustup.rs/). Once you have Rust installed, you can build the project using Cargo, Rust's package manager and build system.

```bash
# Install Rust using rustup (if you haven't already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project in release mode
cargo build --release
```

This will compile the project and produce an executable in the `target/release` directory.

If you'd like to use the `pktbatch` executable system-wide instead of using `cargo run`, you can use the [`install_to_path.sh`](./install_to_path.sh) script to copy the executable to the `/usr/bin/` directory.

```bash
# Install pktbatch to /usr/bin/ (included in $PATH; requires sudo).
./install_to_path.sh

# Now you can use 'pktbatch' from anywhere in the terminal.
sudo pktbatch --help

sudo pktbatch -c ./local.json -i eth0
```

## ⚙️ Usage
After building the project, you can run the executable to generate network packets. The exact usage will depend on the command-line arguments you provide along with the main program's configuration file.

Here is a list of the main command-line arguments you can use.

| Argument | Description |
| --- | --- |
| `-c, --cfg <FILE>` | Path to the configuration file (required) |
| `-l, --list` | Lists the configuration settings and exits. |
| `-w, --watch` | Displays a real-time stats and a graph of the current TX stats of the interface. |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

## First Batch Overrides
You may pass the following flags to override/set the first batch. This is useful for quickly running the tool with different parameters without having to modify the configuration file on disk.

| Argument | Description |
| --- | --- |
| `-i, --iface <OVR_IFACE>` | Override first batch's network interface. |
| `-a, --smac <OVR_SMAC>` | Override first batch's source MAC address. |
| `-b, --dmac <OVR_DMAC>` | Override first batch's destination MAC address. |
| `-s, --src <OVR_SRC_IP>` | Override first batch's source IP address. |
| `-d, --dst <OVR_DST_IP>` | Override first batch's destination IP address. |
| `-p, --protocol <OVR_PROTOCOL>` | Override first batch's protocol. |
| `-q, --sport <OVR_SPORT>` | Override first batch's source port. |
| `-r, --dport <OVR_DPORT>` | Override first batch's destination port. |
| `-n, --threads <OVR_THREAD_CNT>` | Override first batch's thread count. |
| `-I, --interval <OVR_SEND_INTERVAL>` | Override first batch's send interval (microseconds). |
| `-t, --duration <OVR_DURATION>` | Override first batch's duration. |
| `-m, --pl <OVR_PL>` | Override first batch's payload. |
| `-j, --pps <OVR_PPS>` | Override first batch's packets per second. |
| `-k, --bps <OVR_BPS>` | Override first batch's bytes per second. |
| `--wait <OVR_WAIT>` | Override first batch's wait for finish flag. [possible values: true, false] |
| `--max-pkt <OVR_MAX_PKTS>` | Override first batch's maximum packet count. |
| `--max-byt <OVR_MAX_BYTES>` | Override first batch's maximum byte count. |
| `--csum <OVR_CSUM>` | Override first batch's checksum flag. [possible values: true, false] |
| `--l4-csum <OVR_L4_CSUM>` | Override first batch's L4 checksum flag. [possible values: true, false] |
| `--min-ttl <OVR_MIN_TTL>` | Override first batch's minimum TTL. |
| `--max-ttl <OVR_MAX_TTL>` | Override first batch's maximum TTL. |
| `--min-id <OVR_MIN_ID>` | Override first batch's minimum ID. |
| `--max-id <OVR_MAX_ID>` | Override first batch's maximum ID. |
| `--syn <OVR_SYN>` | Override first batch's SYN flag. [possible values: true, false] |
| `--ack <OVR_ACK>` | Override first batch's ACK flag. [possible values: true, false] |
| `--fin <OVR_FIN>` | Override first batch's FIN flag. [possible values: true, false] |
| `--rst <OVR_RST>` | Override first batch's RST flag   [possible values: true, false] |
| `--psh <OVR_PSH>` | Override first batch's PSH flag. [possible values: true, false] |
| `--urg <OVR_URG>` | Override first batch's URG flag. [possible values: true, false] |
| `--ece <OVR_ECE>` | Override first batch's ECE flag. [possible values: true, false] |
| `--cwr <OVR_CWR>` | Override first batch's CWR flag. [possible values: true, false] |
| `--code <OVR_CODE>` | Override first batch's code. |
| `--type <OVR_TYPE>` | Override first batch's type. |
| `--min-len <OVR_MIN_LEN>` | Override first batch's minimum length. |
| `--max-len <OVR_MAX_LEN>` | Override first batch's maximum length. |
| `--static <OVR_IS_STATIC>` | Override first batch's static flag. [possible values: true, false] |
| `--file <OVR_IS_FILE>` | Override first batch's file flag. [possible values: true, false] |
| `--string <OVR_IS_STRING>` | Override first batch's payload's is string flag. [possible values: true, false] |

## 📝 Configuration
The configuration file is a JSON file that defines logging options and the batches of packets to be generated. Each batch can have its own configuration, and the tool will execute each batch sequentially.

The default path is `./config.json`, but you can specify a different path using the `-c` or `--cfg` command-line argument.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `logger` | Logger object | The logging object contain settings related to program and file logging. |
| `tech` | Tech object | The technology object contains settings related to the packet generation technology (e.g., AF_XDP, DPDK, etc.). |
| `batch` | Batch object | The batch object that contains a list of batches and overrides. |

### Logger Object
The logger object contains settings related to `stdout`/`stderr` and on-disk logging.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `level` | Log Level enum (string) | The log level for both console and file logging. |
| `path` | String | `logs/` | The path to the log file or directory. |
| `path_is_file` | Boolean | `false` | Whether the path specified in `path` is a file or a directory. If `false`, log files will be created in the specified directory with names based on the current date and time. If `true`, logs will be written to the specified file. |
| `date_format_file` | String | `"%Y-%m-%d"` | The date format to use for log file names if `path_is_file` is `false`. This uses the same format as the `chrono` crate's `format` method. |
| `date_format_line` | String | `"%Y-%m-%d %H:%M:%S"` | The date format to use for each log line. This uses the same format as the `chrono` crate's `format` method. |

The log level can be set to one of the following values (case-insensitive):
* `trace`
* `debug`
* `info`
* `warn`
* `error`
* `fatal`

### Tech Object
The tech object contains settings related to the packet generation technology (e.g., AF_XDP, DPDK, etc.). This allows you to specify which technology to use for packet generation and any associated settings.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `type` | Tech Type enum (string) | `AfXdp` | The type of technology to use for packet generation. |
| `opts` | Tech Options | `{}` | A JSON object containing options specific to the chosen technology. The available options will depend on the technology type specified in the `type` field. |

The tech type can be set to one of the following values (case-insensitive):
* `af_xdp`, `afxdp`, `af-xdp`

#### AF_XDP Options
Here are the available options for AF_XDP.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `if_name` | String | `None` | The name of the network interface to use for AF_XDP. If the source IP isn't bound to the interface you're using, you will need to set this explicitly. |
| `queue_id` | Integer | `None` | If set, specifies a fixed queue ID to use for **ALL** AF_XDP sockets. |
| `need_wakeup` | Boolean | `false` | Whether to set the need wakeup flag on AF_XDP sockets. |
| `shared_umem` | Boolean | `false` | Whether to use a shared UMEM for all AF_XDP sockets. If `false`, each socket will have its own UMEM. |
| `batch_size` | Integer | `32` | The number of packets to send in each batch when using AF_XDP. This can help improve performance by reducing the number of system calls. |
| `zero_copy` | Boolean | `false` | Whether to bind the AF_XDP sockets in zero-copy mode. |
| `sock_cnt` | Integer | `None` | The number of AF_XDP sockets to use. If set to `0`, the program will use the amount of available CPU cores on the system. |

### Batch Object
The batch object contains a list of batches and overrides. Each batch defines a set of packets to be generated with specific configurations. The tool will execute each batch sequentially, allowing for complex packet generation scenarios (this is also configurable though!).

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `batches` | List of Batch Data objects | `[]` | A list of batches to execute. Each batch will be executed sequentially. |
| `ovr_opts` | Override object | `None` | An option object that contains common settings to override for all batches. |

#### Batch Data Object
Here are the available settings for each batch.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `name` | String | `None` | An optional name to use for the batch when logging/printing information. |
| `iface` | String | `None` | The network interface to use for the batch. This is required if the source IP used isn't bound to the interface you're using. |
| `wait_for_finish` | Boolean | `true` | Whether to wait for the batch to finish before starting the next batch. If `false`, the program will start the next batch immediately after sending all packets for the current batch without waiting for any threads to finish. |
| `max_pkt` | Integer | `None` | The maximum number of packets to send for the batch. If set, this will override the duration settings. |
| `max_byt` | Integer | `None` | The maximum number of bytes to send for the batch. If set, this will override the duration settings. |
| `pps` | Integer | `None` | The target packets per second to send for the batch. This is used to control the sending rate of packets. |
| `bps` | Integer | `None` | The target bytes per second to send for the batch. This is used to control the sending rate of packets. |
| `duration` | Integer | `None` | The duration (in seconds) to run the batch for. |
| `send_interval` | Integer | `1000000` (1 second) | The interval (in microseconds) to wait between sending packets on each thread. This can be used to control the sending rate of packets. `0` will skip the `sleep` call altogether which is best for **max performance**. |
| `thread_cnt` | Integer | `1` | The number of threads to use for sending packets in the batch. Using more threads can help improve performance by allowing for more concurrent packet sending. Using `0` will use the number of available CPU cores. |
| `opt_eth` | Ethernet Options object | `None` | An optional object that contains settings for the Ethernet header. If not set, the program will use default values for the Ethernet header. |
| `opt_ip` | IP Options object | `None` | An optional object that contains settings for the IP header. If not set, the program will use default values for the IP header. |
| `opt_protocol` | Protocol Options object | `None` | An optional object that contains settings for the protocol header (e.g., TCP, UDP, ICMP, etc.). If not set, the program will use default values for the protocol header. |
| `opt_payload` | Payload Options object | `None` | An optional object that contains settings for the packet payload. If not set, the program will use a default payload. |

##### Ethernet Options Object
Here are the available settings for the Ethernet header.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `src_mac` | String | `None` | The source MAC address to use in the Ethernet header. |
| `dst_mac` | String | `None` | The destination MAC address to use in the Ethernet header. |

##### IP Options Object
Here are the available settings for the IP header.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `src` | String | `None` | A single source IP address with CIDR range and DNS resolution support. Examples: `192.168.1.1/24`, `example.com`, `localhost` |
| `srcs` | List of Strings | `None` | Similar to `src`, but allows for specifying multiple source IP addresses. The program will randomly select one of the provided IP addresses for each packet. |
| `dst` | String | `None` | A single destination IP address with CIDR range and DNS resolution support. Examples: `192.168.1.1/24`, `example.com`, `localhost` |
| `dsts` | List of Strings | `None` | Similar to `dst`, but allows for specifying multiple destination IP addresses. The program will randomly select one of the provided IP addresses for each packet. |
| `tos` | Integer | `0` | The Type of Service (TOS) field to use in the IP header. |
| `ttl_min` | Integer | `64` | The minimum Time to Live (TTL) value to use in the IP header. The actual TTL for each packet will be randomly generated between `ttl_min` and `ttl_max`. |
| `ttl_max` | Integer | `64` | The maximum Time to Live (TTL) value to use in the IP header. The actual TTL for each packet will be randomly generated between `ttl_min` and `ttl_max`. |
| `id_min` | Integer | `None` | The minimum ID value to use in the IP header. The actual ID for each packet will be randomly generated between `id_min` and `id_max`. |
| `id_max` | Integer | `None` | The maximum ID value to use in the IP header. The actual ID for each packet will be randomly generated between `id_min` and `id_max`. |
| `do_csum` | Boolean | `true` | Whether to calculate and set the IP header checksum. If `false`, the checksum will be set to `0` (useful for hardware offloading) |

##### Protocol Options Object
The protocol options object contains settings for the protocol header (e.g., TCP, UDP, ICMP, etc.). The available settings will depend on the protocol specified for the batch.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `type` | String | `tcp` | The type of protocol to use for the batch. Supported values include `tcp`, `udp`, and icmp. |
| `opts` | Protocol-specific Options object | `{}` | A JSON object containing options specific to the chosen protocol type. The available options will depend on the protocol type specified in the `type` field. |

###### TCP Options
Here are the available options for the TCP protocol.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `src_port` | Integer | `None` | The source port to use in the TCP header. `0` or `None` randomly generates the port between `1` and `65535`. |
| `dst_port` | Integer | `None` | The destination port to use in the TCP header. `0` or `None` randomly generates the port between `1` and `65535`. |
| `flag_syn` | Boolean | `false` | Whether to set the SYN flag in the TCP header. |
| `flag_ack` | Boolean | `false` | Whether to set the ACK flag in the TCP header. |
| `flag_fin` | Boolean | `false` | Whether to set the FIN flag in the TCP header. |
| `flag_rst` | Boolean | `false` | Whether to set the RST flag in the TCP header. |
| `flag_psh` | Boolean | `false` | Whether to set the PSH flag in the TCP header. |
| `flag_urg` | Boolean | `false` | Whether to set the URG flag in the TCP header. |
| `flag_ece` | Boolean | `false` | Whether to set the ECE flag in the TCP header. |
| `flag_cwr` | Boolean | `false` | | Whether to set the CWR flag in the TCP header. |
| `do_csum` | Boolean | `true` | Whether to calculate and set the TCP header checksum. If `false`, the checksum will be set to `0` (useful for hardware offloading) |

###### UDP Options
Here are the available options for the UDP protocol.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `src_port` | Integer | `None` | The source port to use in the UDP header. `0` or `None` randomly generates the port between `1` and `65535`. |
| `dst_port` | Integer | `None` | The destination port to use in the UDP header. `0` or `None` randomly generates the port between `1` and `65535`. |
| `do_csum` | Boolean | `true` | Whether to calculate and set the UDP header checksum. If `false`, the checksum will be set to `0` (useful for hardware offloading) |

###### ICMP Options
Here are the available options for the ICMP protocol.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `icmp_type` | Integer | `8` (Echo Request) | The ICMP type to use in the ICMP header. |
| `icmp_code` | Integer | `0` | The ICMP code to use in the ICMP header. |
| `do_csum` | Boolean | `true` | Whether to calculate and set the ICMP header checksum. If `false`, the checksum will be set to `0` (useful for hardware offloading) |

#### Payload Options Object
Here are the available settings for the packet payload.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `len_min` | Integer | `None` | The minimum length of the payload in bytes. The actual payload length for each packet will be randomly generated between `len_min` and `len_max`. |
| `len_max` | Integer | `None` | The maximum length of the payload in bytes. The actual payload length for each packet will be randomly generated between `len_min` and `len_max`. |
| `is_static` | Boolean | `false` | Whether to use a static payload for all packets in the batch. If `false`, a random payload will be generated for each packet based on the specified length. |
| `is_file` | Boolean | `false` | Whether to read the `exact` payload from a file. If `true`, the `len_min` and `len_max` fields will be ignored, and the payload will be read from the file specified in the `file_path` field. |
| `is_string` | Boolean | `false` | Whether the `exact` payload is a string. If `true`, the payload will be treated as a UTF-8 string. If `false`, the payload will be treated as raw bytes. |
| `exact` | String | `None` | The exact payload to use for the packets in the batch. The interpretation of this field depends on the values of `is_file` and `is_string`. If `is_file` is `true`, this field is treated as a file path to read the payload from. If `is_string` is `true`, this field is treated as a UTF-8 string to use as the payload. If both `is_file` and `is_string` are `false`, this field is treated as a hex string representing the raw bytes of the payload separated by spaces (e.g. `FF FF FF 04`). |

#### Override Options
Here are the available override options that can be set for all batches.

| Name | Type | Default | Description |
| --- | --- | --- | --- |
| `iface` | String | `None` | Override all batches' network interface. |

## ✍️ Credits
* [Christian Deacon](https://github.com/gamemann)