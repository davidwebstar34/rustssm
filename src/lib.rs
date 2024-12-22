pub mod aws_sso {
    use aws_config::BehaviorVersion;
    use aws_sdk_sso::config::Region;
    use aws_sdk_sso::Client as SsoClient;
    use aws_sdk_ssooidc::Client as SsoOidcClient;
    use dirs_next;
    use skim::prelude::*;
    use std::error::Error;
    use std::fs;
    use std::io::Cursor;
    use tokio::time::{sleep, Duration};
    use webbrowser;

    pub struct AwsSsoWorkflow {
        pub start_url: String,
        pub region: String,
    }

    impl AwsSsoWorkflow {
        fn write_default_aws_credentials(
            access_key_id: &str,
            secret_access_key: &str,
            session_token: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            dirs_next::home_dir()
                .map(|home| home.join(".aws/credentials"))
                .ok_or_else(|| "Could not locate home directory".into())
                .and_then(|credentials_path| {
                    credentials_path
                        .parent()
                        .map(fs::create_dir_all)
                        .transpose()
                        .map_err(|e| e.into())
                        .and_then(|_| {
                            std::fs::write(
                                &credentials_path,
                                format!(
                                    "[default]\naws_access_key_id = {}\naws_secret_access_key = {}\naws_session_token = {}\n",
                                    access_key_id, secret_access_key, session_token
                                ),
                            )
                            .map(|_| {
                                println!("Default credentials written to: {:?}", credentials_path);
                                ()
                            })
                            .map_err(|e| e.into())
                        })
                })
        }

        /// Create a new AWS SSO Workflow instance.
        pub fn new(start_url: String, region: String) -> Self {
            Self { start_url, region }
        }

        /// Main method to execute the workflow.
        pub async fn run(&self) -> Result<(), Box<dyn Error>> {
            // Load AWS configuration
            let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
                .region(Region::new(self.region.clone()))
                .load()
                .await;

            let sso_oidc_client = SsoOidcClient::new(&config);

            // 1. Register the client
            println!("Registering the client...");
            let rc = sso_oidc_client
                .register_client()
                .client_name("my-rust-sso-client")
                .client_type("public")
                .scopes("sso")
                .send()
                .await?;
            let client_id = rc.client_id().expect("Missing client_id");
            let client_secret = rc.client_secret().expect("Missing client_secret");
            println!("Client registered successfully.");

            // 2. Start device authorization
            println!("Starting device authorization...");
            let sda = sso_oidc_client
                .start_device_authorization()
                .client_id(client_id)
                .client_secret(client_secret)
                .start_url(&self.start_url)
                .send()
                .await?;
            let device_code = sda.device_code().expect("Missing device_code");
            let user_code = sda.user_code().expect("Missing user_code");
            let verification_uri = sda.verification_uri().expect("Missing verification_uri");
            let verification_uri_complete = sda
                .verification_uri_complete()
                .expect("Missing verificationUriComplete");
            let interval = sda.interval();
            println!("Device authorization started successfully.");

            // Open browser to verification URI
            println!("Opening the verification page in your browser...");
            if webbrowser::open(verification_uri_complete).is_ok() {
                println!("Browser successfully opened. Please authenticate to continue.");
            } else {
                println!(
                    "Could not open the browser. Please go to: {}",
                    verification_uri
                );
                println!("Enter the user code: {}", user_code);
            }

            // Wait for user to complete browser authentication
            println!("Please complete the authentication in your browser.");
            println!("Press Enter after you have completed the process...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            // 3. Poll for token using a loop
            println!("Polling for token...");
            let token_response = loop {
                match sso_oidc_client
                    .create_token()
                    .client_id(client_id)
                    .client_secret(client_secret)
                    .grant_type("urn:ietf:params:oauth:grant-type:device_code")
                    .device_code(device_code)
                    .send()
                    .await
                {
                    Ok(tr) => {
                        println!("Token received successfully.");
                        break Some(tr); // Exit loop with token
                    }
                    Err(e) => {
                        let msg = format!("{}", e);
                        if msg.contains("authorization_pending") {
                            println!("Authorization pending, retrying in {} seconds...", interval);
                            sleep(Duration::from_secs(interval as u64)).await;
                        } else if msg.contains("slow_down") {
                            println!(
                                "Too many requests. Slowing down, retrying in {} seconds...",
                                interval + 5
                            );
                            sleep(Duration::from_secs(interval as u64 + 5)).await;
                        } else {
                            eprintln!("Error: CreateToken failed with message: {}", msg);
                            return Err(format!("CreateToken failed: {}", msg).into());
                        }
                    }
                }
            };

            let token_response =
                token_response.ok_or("Timed out waiting for user authorization")?;
            let access_token = token_response
                .access_token()
                .ok_or("Missing access_token")?;
            println!("Access token retrieved successfully.");

            // 4. Use the access token to list accounts and roles
            let sso_client = SsoClient::new(&config);
            println!("Fetching accounts and roles...");
            let accounts_resp = sso_client
                .list_accounts()
                .access_token(access_token)
                .send()
                .await?;

            let accounts = accounts_resp.account_list(); // Directly get the slice

            if accounts.is_empty() {
                println!("No accounts found.");
                return Ok(());
            }

            let mut account_role_strings = Vec::new();

            for account in accounts {
                if let Some(account_id) = account.account_id() {
                    let account_name = account.account_name().unwrap_or("Unknown");

                    // Fetch roles for each account
                    let roles_resp = sso_client
                        .list_account_roles()
                        .account_id(account_id)
                        .access_token(access_token)
                        .send()
                        .await?;

                    // Format account and roles
                    for role in roles_resp.role_list() {
                        if let Some(role_name) = role.role_name() {
                            account_role_strings
                                .push(format!("{} - {} - {}", account_id, account_name, role_name));
                        }
                    }
                }
            }

            // Check if no roles were found
            if account_role_strings.is_empty() {
                println!("No accounts or roles found.");
                return Ok(());
            }

            // 3. Use `skim` for fuzzy search
            let options = SkimOptionsBuilder::default()
                .height(Some("20"))
                .multi(true)
                .prompt(Some("Select accounts and roles: "))
                .build()
                .unwrap();

            let item_reader = SkimItemReader::default();
            let items = item_reader.of_bufread(Cursor::new(account_role_strings.join("\n")));
            let selected_items = Skim::run_with(&options, Some(items))
                .map(|out| out.selected_items)
                .unwrap_or_else(|| Vec::new());

            // 4. Handle selected accounts
            if selected_items.is_empty() {
                println!("No accounts selected.");
                return Ok(());
            }

            for item in selected_items {
                // Parse the selected item
                let selected_output = item.output();
                println!("Selected account and role: {}", selected_output);

                // Split the selected item into account_id, account_name, and role_name
                let parts: Vec<&str> = selected_output.split(" - ").collect();
                if parts.len() != 3 {
                    eprintln!("Invalid selection format: {}", selected_output);
                    continue;
                }
                let account_id = parts[0];
                let role_name = parts[2];

                println!(
                    "Fetching credentials for Account ID: {}, Role: {}",
                    account_id, role_name
                );

                // Fetch credentials for the selected role
                let credentials_resp = sso_client
                    .get_role_credentials()
                    .account_id(account_id)
                    .role_name(role_name)
                    .access_token(access_token)
                    .send()
                    .await?;

                if let Some(credentials) = credentials_resp.role_credentials() {
                    let access_key_id = credentials.access_key_id().unwrap_or("");
                    let secret_access_key = credentials.secret_access_key().unwrap_or("");
                    let session_token = credentials.session_token().unwrap_or("");

                    println!("Environment variables updated for the selected role.");
                    Self::write_default_aws_credentials(
                        access_key_id,
                        secret_access_key,
                        session_token,
                    )?;
                } else {
                    eprintln!(
                        "Failed to fetch credentials for Account ID: {}, Role: {}",
                        account_id, role_name
                    );
                }
            }

            println!("Process completed successfully.");

            Ok(())
        }
    }
}
