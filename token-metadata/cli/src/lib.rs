use clap::ArgEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, ArgEnum)]
pub enum SolanaUrl {
    #[default]
    Localnet,
    Devnet,
}

impl SolanaUrl {
    pub fn url(&self) -> url::Url {
        match self {
            SolanaUrl::Localnet => url::Url::parse("http://127.0.0.1:8899/").unwrap(),
            SolanaUrl::Devnet => url::Url::parse("https://api.devnet.solana.com/").unwrap(),
        }
    }
}

impl std::fmt::Display for SolanaUrl {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            SolanaUrl::Localnet => f.write_str("localnet"),
            SolanaUrl::Devnet => f.write_str("devnet"),
        }
    }
}
