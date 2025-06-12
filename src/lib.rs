use pyo3::prelude::*;
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::Value;
use dashmap::DashMap;
use thiserror::Error;
use tokio::runtime::{Builder, Runtime};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}

#[derive(Parser)]
#[command(name = "coinwagon", about = "A cryptocurrency CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    CurrentPrice {
        #[arg(help = "Cryptocurrency symbol (e.g., bitcoin)")]
        crypto: String,
        #[arg(help = "Fiat currency symbol (e.g., usd)")]
        fiat: String,
        #[arg(long, help = "Enable verbose output")]
        verbose: bool,
    },
    AddressBalance {
        #[arg(help = "Cryptocurrency symbol (e.g., bitcoin)")]
        crypto: String,
        #[arg(help = "Wallet address")]
        address: String,
        #[arg(long, help = "Enable verbose output")]
        verbose: bool,
    },
    WalletBalance {
        #[arg(help = "Path to wallet file (crypto,address per line)")]
        wallet: String,
        #[arg(help = "Fiat currency symbol (e.g., usd)")]
        fiat: String,
        #[arg(long, help = "Enable verbose output")]
        verbose: bool,
    },
}

struct CryptoTool {
    client: Client,
    cache: Arc<DashMap<String, (f64, SystemTime)>>,
    cache_ttl: Duration,
}

impl CryptoTool {
    fn new() -> Self {
        CryptoTool {
            client: Client::new(),
            cache: Arc::new(DashMap::new()),
            cache_ttl: Duration::from_secs(300), // 5-minute TTL
        }
    }

    async fn get_current_price(&self, crypto: &str, fiat: &str, verbose: bool) -> Result<f64, CryptoError> {
        let key = format!("{}_{}", crypto, fiat);
        if let Some(entry) = self.cache.get(&key) {
            let (price, timestamp) = *entry;
            if SystemTime::now().duration_since(timestamp)? < self.cache_ttl {
                if verbose {
                    println!("Using cached price for {}/{}", crypto, fiat);
                }
                return Ok(price);
            }
        }

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
            crypto, fiat
        );
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        let price = resp[crypto][fiat]
            .as_f64()
            .ok_or_else(|| CryptoError::ApiError("Invalid price response".to_string()))?;

        self.cache.insert(key, (price, SystemTime::now()));
        if verbose {
            println!("Fetched price from CoinGecko: {}", price);
        }
        Ok(price)
    }

    async fn get_address_balance(&self, crypto: &str, address: &str, verbose: bool) -> Result<f64, CryptoError> {
        // Try BlockCypher API first (more reliable)
        if crypto == "bitcoin" {
            let url = format!("https://api.blockcypher.com/v1/btc/main/addrs/{}/balance", address);
            if let Ok(resp) = self.client.get(&url).send().await {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(balance) = json["balance"].as_u64() {
                        let btc_balance = balance as f64 / 100_000_000.0; // Convert satoshis to BTC
                        if verbose {
                            println!("Fetched balance from BlockCypher: {} BTC", btc_balance);
                        }
                        return Ok(btc_balance);
                    }
                }
            }
        }

        // Fallback to Blockchair API with improved error handling
        let url = format!("https://api.blockchair.com/{}/dashboards/address/{}", crypto, address);
        let resp = self.client.get(&url).send().await?;
        let json: Value = resp.json().await?;
        
        if verbose {
            println!("API Response: {}", serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Invalid JSON".to_string()));
        }
        
        // Try different possible response structures
        let balance = if let Some(data) = json.get("data") {
            if let Some(addr_data) = data.get(address) {
                if let Some(address_info) = addr_data.get("address") {
                    address_info.get("balance").and_then(|v| v.as_f64())
                } else {
                    addr_data.get("balance").and_then(|v| v.as_f64())
                }
            } else {
                None
            }
        } else {
            json.get("balance").and_then(|v| v.as_f64())
        };

        match balance {
            Some(bal) => {
                let btc_balance = bal / 100_000_000.0; // Convert satoshis to BTC
                if verbose {
                    println!("Fetched balance from Blockchair: {} {}", btc_balance, crypto.to_uppercase());
                }
                Ok(btc_balance)
            },
            None => Err(CryptoError::ApiError(format!("Could not parse balance from response: {}", 
                serde_json::to_string(&json).unwrap_or_else(|_| "Invalid JSON".to_string()))))
        }
    }

    async fn get_wallet_balance(&self, wallet: &str, fiat: &str, verbose: bool) -> Result<Vec<(String, f64, f64, f64)>, CryptoError> {
        let wallets = std::fs::read_to_string(wallet)
            .map_err(|e| CryptoError::InvalidInput(format!("Failed to read wallet file: {}", e)))?
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() != 2 {
                    Err(CryptoError::InvalidInput(format!("Invalid wallet line: {}", line)))
                } else {
                    Ok((parts[0].to_string(), parts[1].trim().to_string()))
                }
            })
            .collect::<Result<Vec<_>, CryptoError>>()?;

        let mut results = Vec::new();
        for (crypto, address) in wallets {
            let balance = self.get_address_balance(&crypto, &address, verbose).await?;
            let price = self.get_current_price(&crypto, fiat, verbose).await?;
            let fiat_value = balance * price;
            results.push((crypto.to_uppercase(), balance, fiat_value, price));
        }
        Ok(results)
    }
}

#[pyfunction]
unsafe fn run_command(_command: String, args: Vec<String>) -> PyResult<String> {
    unsafe {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Runtime error: {}", e)))?;
        let tool = CryptoTool::new();
        
        let cli = Cli::try_parse_from(std::iter::once("coinwagon".to_string())
            .chain(std::iter::once(_command))
            .chain(args.into_iter()))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid arguments: {}", e)))?;

        let result = rt.block_on(async {
            match cli.command {
                Commands::CurrentPrice { crypto, fiat, verbose } => {
                    match tool.get_current_price(&crypto, &fiat, verbose).await {
                        Ok(price) => Ok(format!("{} {}", price, fiat.to_uppercase())),
                        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Error: {}", e))),
                    }
                }
                Commands::AddressBalance { crypto, address, verbose } => {
                    match tool.get_address_balance(&crypto, &address, verbose).await {
                        Ok(balance) => Ok(format!("{} {}", balance, crypto.to_uppercase())),
                        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Error: {}", e))),
                    }
                }
                Commands::WalletBalance { wallet, fiat, verbose } => {
                    match tool.get_wallet_balance(&wallet, &fiat, verbose).await {
                        Ok(results) => {
                            let mut output = String::new();
                            let mut total_fiat = 0.0;
                            for (crypto, balance, fiat_value, price) in results {
                                output.push_str(&format!(
                                    "{}: {} {} = {} {}\n",
                                    crypto, balance, crypto, fiat_value, fiat.to_uppercase()
                                ));
                                total_fiat += fiat_value;
                            }
                            output.push_str(&format!("Total: {} {}", total_fiat, fiat.to_uppercase()));
                            Ok(output)
                        }
                        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Error: {}", e))),
                    }
                }
            }
        })?;
        Ok(result)
    }
}

#[pymodule]
fn coinwagon(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_command, m)?)?;
    Ok(())
}