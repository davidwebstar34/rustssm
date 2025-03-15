# rustssm

A Rust-based AWS SSM session helper for seamless SSH access and session management with AWS EC2 instances.

`rustssm` simplifies securely interacting with your EC2 instances through AWS Systems Manager (SSM) without needing direct SSH access or managing SSH keys manually.

---

## 🚀 Features

- **Interactive EC2 instance selection** (powered by `inquire`).
- **Automatic SSH key installation** via AWS SSM.
- **Secure interactive shell access** to EC2 instances without SSH.
- **Easy local SSH tunneling** using AWS Session Manager (port forwarding).
- **Start Jupyter Notebooks remotely** via AWS SSM with automatic port forwarding.

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

### CLI Commands

`rustssm` supports different actions via CLI commands:

**Interactive Connect**

```sh
rustssm connect
```

- Choose an EC2 instance interactively.
- Automatically establish an interactive shell session.

**Copy SSH Key**

```sh
rustssm copy-key --ssh-key-path ~/.ssh/id_rsa.pub --username ssm-user
```

- Copies your local SSH public key to the remote EC2 instance.

**SSH Tunnel (Port Forwarding)**

```sh
rustssm tunnel --local-port 2222 --remote-port 22
```

- Establishes a secure SSH tunnel to your EC2 instance.

Then, in another terminal:

```sh
ssh -i ~/.ssh/id_rsa -p 2222 ssm-user@localhost
```

**Remote Jupyter Notebook**

```sh
rustssm notebook --username ubuntu --local-port 8888
```

- Starts a Jupyter Notebook remotely and forwards it locally.

Access Jupyter at:

```
http://localhost:8888
```

---

## 📌 Prerequisites

- [AWS Session Manager Plugin](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html) installed.
- AWS credentials configured (`~/.aws/credentials`).
- Your EC2 instances must have the AWS SSM agent installed.
- Proper IAM permissions for SSM (`ssm:StartSession`, `ssm:SendCommand`, `ssm:TerminateSession`).

---

## 🛠️ Developing & Extending

Feel free to clone and modify the tool to your needs:

```sh
git clone https://your.repo.url/rustssm.git
cd rustssm
cargo build
```

Contributions are welcome! 🎉

---

## 🔍 Troubleshooting

- **Invalid Operation Errors:** Check JSON arguments formatting for `session-manager-plugin`.
- **SSH key not appearing:** Ensure the correct instance username (`ubuntu`, `ec2-user`, or `ssm-user`) is specified.

---

## 📝 License

Licensed under MIT. Enjoy!
