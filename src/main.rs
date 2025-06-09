use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenUrl, AuthorizationCode, PkceCodeVerifier,
};
use std::env;
use std::io;
use std::path::PathBuf;
use zoom_video_mover_lib::{Config, ZoomRecordingDownloader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zoom Cloud Recording Downloader");
    println!("================================");

    let access_token = match env::var("ZOOM_ACCESS_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            println!("ZOOM_ACCESS_TOKEN not found. Starting OAuth flow...");
            get_access_token().await?
        }
    };

    let user_id = env::var("ZOOM_USER_ID").unwrap_or_else(|_| "me".to_string());
    
    println!("Enter date range for recordings:");
    print!("From date (YYYY-MM-DD): ");
    let mut from_date = String::new();
    io::stdin().read_line(&mut from_date)?;
    let from_date = from_date.trim();

    print!("To date (YYYY-MM-DD): ");
    let mut to_date = String::new();
    io::stdin().read_line(&mut to_date)?;
    let to_date = to_date.trim();

    print!("Output directory (default: ./downloads): ");
    let mut output_dir = String::new();
    io::stdin().read_line(&mut output_dir)?;
    let output_dir = if output_dir.trim().is_empty() {
        get_default_downloads_dir()
    } else {
        output_dir.trim().to_string()
    };

    let downloader = ZoomRecordingDownloader::new(access_token);
    
    println!("Fetching recordings from {} to {}...", from_date, to_date);
    
    match downloader.download_all_recordings(&user_id, from_date, to_date, &output_dir).await {
        Ok(files) => {
            println!("\nDownload completed!");
            println!("Downloaded {} files to {}", files.len(), output_dir);
            for file in files {
                println!("  - {}", file);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = "config.toml";
    
    match Config::load_from_file(config_path) {
        Ok(config) => {
            if config.client_id == "your_zoom_client_id" || config.client_secret == "your_zoom_client_secret" {
                return Err("Please update config.toml with your actual Zoom API credentials".into());
            }
            Ok(config)
        }
        Err(_) => {
            println!("Config file not found. Creating sample config.toml...");
            Config::create_sample_file(config_path)?;
            Err("Created sample config.toml. Please update it with your Zoom API credentials and run again.".into())
        }
    }
}

async fn get_access_token() -> Result<String, Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let redirect_uri = config.redirect_uri.unwrap_or_else(|| "http://localhost:8080/callback".to_string());
    
    let oauth_client = BasicClient::new(
        ClientId::new(config.client_id),
        Some(ClientSecret::new(config.client_secret)),
        AuthUrl::new("https://zoom.us/oauth/authorize".to_string())?,
        Some(TokenUrl::new("https://zoom.us/oauth/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("recording:read".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("Please visit this URL to authorize the application:");
    println!("{}", auth_url);
    
    #[cfg(windows)]
    {
        println!("Opening URL in default browser...");
        open_url_windows(&auth_url.to_string());
    }
    
    println!("\nAfter authorization, copy the authorization code from the URL:");
    
    let mut auth_code = String::new();
    io::stdin().read_line(&mut auth_code)?;
    let auth_code = auth_code.trim();

    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    let access_token = token_result.access_token().secret().to_string();
    
    println!("Access token obtained! You can set it as an environment variable:");
    if cfg!(windows) {
        println!("set ZOOM_ACCESS_TOKEN={}", access_token);
    } else {
        println!("export ZOOM_ACCESS_TOKEN={}", access_token);
    }
    
    Ok(access_token)
}

fn get_default_downloads_dir() -> String {
    if cfg!(windows) {
        match dirs::download_dir() {
            Some(path) => path.join("ZoomRecordings").to_string_lossy().to_string(),
            None => ".\\downloads".to_string(),
        }
    } else {
        "./downloads".to_string()
    }
}

#[cfg(windows)]
fn open_url_windows(url: &str) {
    use std::process::Command;
    let _ = Command::new("cmd")
        .args(&["/C", "start", url])
        .spawn();
}