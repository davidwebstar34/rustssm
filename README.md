# rustssm

A Rust-based AWS SSM session helper for seamless SSH access and session management with AWS EC2 instances.

`rustssm` simplifies securely interacting with your EC2 instances through AWS Systems Manager (SSM) without needing direct SSH access or managing SSH keys manually.

---

## 🚀 Features

- **Interactive EC2 instance selection** (powered by `inquire`).
- **Automatic SSH key installation** via AWS SSM.
- **Secure interactive shell access** to EC2 instances without SSH.
- **Easy local SSH tunneling** using AWS Session Manager (port forwarding).

---

## 📥 Installation

### Using Pre-built Binaries (Recommended)

Download the latest binary for your OS using `curl`:

**macOS**

```sh
curl -L -o rustssm https://github.com/yourusername/rustssm/releases/latest/download/rustssm-x86_64-apple-darwin
chmod +x rustssm
sudo mv rustssm /usr/local/bin/
xattr -d com.apple.quarantine /usr/local/bin/rustssm
```

**Linux**

```sh
curl -L -o rustssm https://github.com/yourusername/rustssm/releases/latest/download/rustssm-x86_64-unknown-linux-gnu
chmod +x rustssm
sudo mv rustssm /usr/local/bin/
```

### Using Cargo

Clone this repository, then build and install:

```sh
cargo build --release
sudo cp target/release/rustssm /usr/local/bin/
```

---

## 🎯 Usage

### Basic interactive use

```sh
rustssm
```

- Choose an EC2 instance interactively.
- Automatically install your local SSH public key.
- Establish an interactive shell session.

### Port forwarding (SSH tunneling)

Create an SSH tunnel (local port `2222` → remote port `22`):

```sh
rustssm
```

Then, in another terminal:

```sh
ssh -i ~/.ssh/id_rsa -p 2222 ssm-user@localhost
```
