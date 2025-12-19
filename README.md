# RustOTP

A Rust-based One-Time Password (OTP) code viewer displaying TOTP authentication codes.

## Features

- Generate Time-based One-Time Passwords (TOTP)
- Command-line interface for easy usage
- Support for standard OTP parameters

## Installation

```bash
git clone https://github.com/yourusername/rustotp.git
cd rustotp
cargo build --release
```

## Usage

Fill `./rustotp.txt` file with
```txt
name=code
name2=code
```

and run the application.
```

## Dependencies

- `totp-rs` or similar OTP library
- `clap` for command-line argument parsing

## License

MIT License