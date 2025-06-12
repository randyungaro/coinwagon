
![08ddf93e-273d-4b4f-ad20-5beac9a8ee87---](https://github.com/user-attachments/assets/071d53e5-3677-4a18-8df1-01fdaf6c67f0)


# 🪙 Coinwagon


A fast, reliable Python package for cryptocurrency operations built with Rust.
Get real-time prices, check wallet balances, and manage multiple addresses with ease.
Which inspired and implement the main features of [Moneywagon](https://github.com/priestc/moneywagon)

## ✨ Features

- 🚀 **Fast**: Built with Rust for maximum performance
- 💰 **Real-time Prices**: Get current cryptocurrency prices in multiple fiat currencies
- 🏦 **Address Balance**: Check balance of individual cryptocurrency addresses
- 📊 **Wallet Management**: Manage and calculate total value of multiple addresses
- 🔄 **Smart Caching**: Built-in caching with configurable TTL (5-minute default)
- 🌐 **Multiple APIs**: Uses BlockCypher and Blockchair APIs with automatic fallback
- 📝 **Verbose Mode**: Detailed logging for debugging and monitoring
- 🔒 **Error Handling**: Comprehensive error handling with descriptive messages

## 🚀 Installation

### From PyPI
```bash
pip install coinwagon
```

### From Source
```bash
git clone https://github.com/randyungaro/coinwagon.git
cd coinwagon
pip install maturin
maturin develop
```

## 📖 Quick Start

```python
import coinwagon

# Get Bitcoin price in USD
price = coinwagon.run_command("current-price", ["bitcoin", "usd"])
print(price)  # Output: "67234.50 USD"

# Check Bitcoin address balance
balance = coinwagon.run_command("address-balance", ["bitcoin", "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"])
print(balance)  # Output: "68.91234567 BITCOIN"

# Check wallet balance from file
result = coinwagon.run_command("wallet-balance", ["my_wallet.txt", "usd"])
print(result)
# Output:
# BITCOIN: 1.50000000 BITCOIN = 100851.75 USD
# BITCOIN: 0.25000000 BITCOIN = 16808.63 USD
# Total: 117660.38 USD
```

## 🛠️ Usage

### 1. Get Current Cryptocurrency Price

```python
import coinwagon

# Basic usage
price = coinwagon.run_command("current-price", ["bitcoin", "usd"])
print(f"Bitcoin price: {price}")

# With verbose output
price = coinwagon.run_command("current-price", ["bitcoin", "usd", "--verbose"])

# Different currencies
eur_price = coinwagon.run_command("current-price", ["bitcoin", "eur"])
jpy_price = coinwagon.run_command("current-price", ["ethereum", "jpy"])
```

### 2. Check Address Balance

```python
import coinwagon

# Check Bitcoin address
btc_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
balance = coinwagon.run_command("address-balance", ["bitcoin", btc_address])
print(f"Balance: {balance}")

# With verbose output for debugging
balance = coinwagon.run_command("address-balance", ["bitcoin", btc_address, "--verbose"])
```

### 3. Wallet Balance Management

Create a wallet file (`my_wallet.txt`):
```
# My cryptocurrency addresses
bitcoin,1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
bitcoin,1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2
bitcoin,3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy
```

Check total wallet value:
```python
import coinwagon

# Get total wallet value in USD
result = coinwagon.run_command("wallet-balance", ["my_wallet.txt", "usd"])
print(result)

# With verbose output
result = coinwagon.run_command("wallet-balance", ["my_wallet.txt", "usd", "--verbose"])
```

## 📋 Command Reference

### `current-price`
Get real-time cryptocurrency prices.

**Usage:** `coinwagon.run_command("current-price", [crypto, fiat, "--verbose"])`

**Parameters:**
- `crypto`: Cryptocurrency symbol (e.g., "bitcoin", "ethereum")
- `fiat`: Fiat currency symbol (e.g., "usd", "eur", "jpy")
- `--verbose`: Optional flag for detailed output

### `address-balance`
Check balance of a specific cryptocurrency address.

**Usage:** `coinwagon.run_command("address-balance", [crypto, address, "--verbose"])`

**Parameters:**
- `crypto`: Cryptocurrency symbol (e.g., "bitcoin")
- `address`: Wallet address to check
- `--verbose`: Optional flag for detailed output

### `wallet-balance`
Calculate total value of multiple addresses from a file.

**Usage:** `coinwagon.run_command("wallet-balance", [wallet_file, fiat, "--verbose"])`

**Parameters:**
- `wallet_file`: Path to wallet file (format: `crypto,address` per line)
- `fiat`: Fiat currency for total calculation
- `--verbose`: Optional flag for detailed output

## 📁 Wallet File Format

Create a text file with one address per line in the format `crypto,address`:

```
# Comments start with #
bitcoin,1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
bitcoin,1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2
ethereum,0x742d35Cc6634C0532925a3b8D4Fddfac9e2C4cb7

# Empty lines are ignored
bitcoin,3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy
```

## 🔧 Advanced Usage

### Error Handling

```python
import coinwagon

try:
    balance = coinwagon.run_command("address-balance", ["bitcoin", "invalid_address"])
    print(balance)
except ValueError as e:
    print(f"Invalid arguments: {e}")
except RuntimeError as e:
    print(f"Runtime error: {e}")
```

### Batch Operations

```python
import coinwagon

addresses = [
    "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
    "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"
]

for addr in addresses:
    try:
        balance = coinwagon.run_command("address-balance", ["bitcoin", addr])
        print(f"{addr}: {balance}")
    except Exception as e:
        print(f"Error checking {addr}: {e}")
```


## 🔍 Supported Cryptocurrencies

- **Bitcoin** (`bitcoin`)
- **Ethereum** (`ethereum`) 
- **Litecoin** (`litecoin`)
- And many more supported by CoinGecko API

For the complete list, check the [CoinGecko API documentation](https://www.coingecko.com/en/api/documentation).

## ⚡ Performance

- **Caching**: Automatic caching with 5-minute TTL reduces API calls
- **Async Operations**: Built on Tokio for non-blocking I/O
- **Rust Performance**: Core operations written in Rust for maximum speed
- **Multiple APIs**: Automatic fallback ensures reliability

## 🐛 Troubleshooting

### Common Issues

1. **API Rate Limits**: If you encounter rate limit errors, the built-in caching will help reduce API calls.

2. **Invalid Address Format**: Make sure cryptocurrency addresses are in the correct format for the specific blockchain.

3. **Network Issues**: The package will retry failed requests and use fallback APIs when possible.

### Debug Mode

Use the `--verbose` flag to see detailed information about API calls:

```python
result = coinwagon.run_command("address-balance", ["bitcoin", "address", "--verbose"])
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.


## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


## 🙋‍♂️ Support

Buy me Cup of Coffee

BTC : bc1qxk3enn909extqfp57fvgfgve5xcw66cusd04se

---
