use clap::Parser;

use crate::config::base::DEFAULT_CONFIG_PATH;

#[derive(Parser, Default, Clone)]
#[clap(version, about, long_about = None)]
pub struct Args {
    #[clap(short = 'c', long = "cfg", default_value = DEFAULT_CONFIG_PATH, help = "Path to configuration file.")]
    pub config: String,

    #[clap(
        short = 'l',
        long = "list",
        help = "Lists the configuration settings and exits."
    )]
    pub list_cfg: bool,

    #[clap(
        short = 'w',
        long = "watch",
        help = "Displays a real-time stats and a graph of the current TX stats of the interface."
    )]
    pub watch: bool,

    /* First batch overrides */
    // Primary/popular overrides (includes short options)
    #[clap(
        short = 'i',
        long = "iface",
        help = "Override first batch's network interface."
    )]
    pub ovr_iface: Option<String>,

    #[clap(
        short = 'a',
        long = "smac",
        help = "Override first batch's source MAC address."
    )]
    pub ovr_smac: Option<String>,

    #[clap(
        short = 'b',
        long = "dmac",
        help = "Override first batch's destination MAC address."
    )]
    pub ovr_dmac: Option<String>,

    #[clap(
        short = 's',
        long = "src",
        help = "Override first batch's source IP address."
    )]
    pub ovr_src_ip: Option<String>,

    #[clap(
        short = 'd',
        long = "dst",
        help = "Override first batch's destination IP address."
    )]
    pub ovr_dst_ip: Option<String>,

    #[clap(
        short = 'p',
        long = "protocol",
        help = "Override first batch's protocol."
    )]
    pub ovr_protocol: Option<String>,

    #[clap(
        short = 'q',
        long = "sport",
        help = "Override first batch's source port."
    )]
    pub ovr_sport: Option<u16>,

    #[clap(
        short = 'r',
        long = "dport",
        help = "Override first batch's destination port."
    )]
    pub ovr_dport: Option<u16>,

    #[clap(
        short = 'n',
        long = "threads",
        help = "Override first batch's thread count."
    )]
    pub ovr_thread_cnt: Option<u32>,

    #[clap(
        short = 'I',
        long = "interval",
        help = "Override first batch's send interval (microseconds)."
    )]
    pub ovr_send_interval: Option<u64>,

    #[clap(
        short = 't',
        long = "duration",
        help = "Override first batch's duration."
    )]
    pub ovr_duration: Option<u32>,

    #[clap(short = 'm', long = "pl", help = "Override first batch's payload.")]
    pub ovr_pl: Option<String>,

    #[clap(
        short = 'j',
        long = "pps",
        help = "Override first batch's packets per second."
    )]
    pub ovr_pps: Option<u32>,

    #[clap(
        short = 'k',
        long = "bps",
        help = "Override first batch's bytes per second."
    )]
    pub ovr_bps: Option<u64>,

    // Additional overrides (normally not associated with short options)
    #[clap(long = "wait", help = "Override first batch's wait for finish flag.")]
    pub ovr_wait: Option<bool>,

    #[clap(
        long = "max-pkt",
        help = "Override first batch's maximum packet count."
    )]
    pub ovr_max_pkts: Option<u32>,

    #[clap(long = "max-byt", help = "Override first batch's maximum byte count.")]
    pub ovr_max_bytes: Option<u32>,

    #[clap(long = "csum", help = "Override first batch's checksum flag.")]
    pub ovr_csum: Option<bool>,

    #[clap(long = "l4-csum", help = "Override first batch's L4 checksum flag.")]
    pub ovr_l4_csum: Option<bool>,

    #[clap(long = "min-ttl", help = "Override first batch's minimum TTL.")]
    pub ovr_min_ttl: Option<u8>,

    #[clap(long = "max-ttl", help = "Override first batch's maximum TTL.")]
    pub ovr_max_ttl: Option<u8>,

    #[clap(long = "min-id", help = "Override first batch's minimum ID.")]
    pub ovr_min_id: Option<u16>,

    #[clap(long = "max-id", help = "Override first batch's maximum ID.")]
    pub ovr_max_id: Option<u16>,

    #[clap(long = "syn", help = "Override first batch's SYN flag.")]
    pub ovr_syn: Option<bool>,

    #[clap(long = "ack", help = "Override first batch's ACK flag.")]
    pub ovr_ack: Option<bool>,

    #[clap(long = "fin", help = "Override first batch's FIN flag.")]
    pub ovr_fin: Option<bool>,

    #[clap(long = "rst", help = "Override first batch's RST flag.")]
    pub ovr_rst: Option<bool>,

    #[clap(long = "psh", help = "Override first batch's PSH flag.")]
    pub ovr_psh: Option<bool>,

    #[clap(long = "urg", help = "Override first batch's URG flag.")]
    pub ovr_urg: Option<bool>,

    #[clap(long = "ece", help = "Override first batch's ECE flag.")]
    pub ovr_ece: Option<bool>,

    #[clap(long = "cwr", help = "Override first batch's CWR flag.")]
    pub ovr_cwr: Option<bool>,

    #[clap(long = "code", help = "Override first batch's code.")]
    pub ovr_code: Option<u8>,

    #[clap(long = "type", help = "Override first batch's type.")]
    pub ovr_type: Option<u8>,

    #[clap(long = "min-len", help = "Override first batch's minimum length.")]
    pub ovr_min_len: Option<u16>,

    #[clap(long = "max-len", help = "Override first batch's maximum length.")]
    pub ovr_max_len: Option<u16>,

    #[clap(long = "static", help = "Override first batch's static flag.")]
    pub ovr_is_static: Option<bool>,

    #[clap(long = "file", help = "Override first batch's file flag.")]
    pub ovr_is_file: Option<bool>,

    #[clap(
        long = "string",
        help = "Override first batch's payload's is string flag."
    )]
    pub ovr_is_string: Option<bool>,
}
