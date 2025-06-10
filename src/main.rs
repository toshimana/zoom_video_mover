use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenUrl, AuthorizationCode, TokenResponse,
};
use std::env;
use std::io;
use zoom_video_mover_lib::{Config, ZoomRecordingDownloader, windows_console};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Windows環境でのコンソール文字化け対策
    windows_console::setup_console_encoding();
    
    windows_console::println_japanese("Zoom録画ダウンローダー");
    windows_console::println_japanese("====================");

    let access_token = match env::var("ZOOM_ACCESS_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            windows_console::println_japanese("ZOOM_ACCESS_TOKEN が見つかりません。OAuth認証を開始します...");
            get_access_token().await?
        }
    };

    let user_id = env::var("ZOOM_USER_ID").unwrap_or_else(|_| "me".to_string());
    
    windows_console::println_japanese("録画の日付範囲を入力してください:");
    windows_console::print_japanese("開始日 (YYYY-MM-DD): ");
    let mut from_date = String::new();
    io::stdin().read_line(&mut from_date)?;
    let from_date = from_date.trim();

    windows_console::print_japanese("終了日 (YYYY-MM-DD): ");
    let mut to_date = String::new();
    io::stdin().read_line(&mut to_date)?;
    let to_date = to_date.trim();

    windows_console::print_japanese("保存先ディレクトリ (デフォルト: ./downloads): ");
    let mut output_dir = String::new();
    io::stdin().read_line(&mut output_dir)?;
    let output_dir = if output_dir.trim().is_empty() {
        get_default_downloads_dir()
    } else {
        output_dir.trim().to_string()
    };

    let downloader = ZoomRecordingDownloader::new(access_token);
    
    windows_console::println_japanese(&format!("{}から{}までの録画を取得中...", from_date, to_date));
    
    match downloader.download_all_recordings(&user_id, from_date, to_date, &output_dir).await {
        Ok(files) => {
            windows_console::println_japanese("\nダウンロード完了!");
            windows_console::println_japanese(&format!("{}個のファイルを{}にダウンロードしました", files.len(), output_dir));
            for file in files {
                windows_console::println_japanese(&format!("  - {}", file));
            }
        }
        Err(e) => {
            windows_console::println_japanese(&format!("エラー: {}", e));
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
                return Err("config.tomlを実際のZoom API認証情報で更新してください".into());
            }
            
            // Validate Client ID format (should start with specific characters for Zoom)
            if config.client_id.is_empty() || config.client_secret.is_empty() {
                return Err("Client IDとClient Secretは空にできません".into());
            }
            
            windows_console::println_japanese(&format!("使用するClient ID: {}...", &config.client_id[..std::cmp::min(8, config.client_id.len())]));
            Ok(config)
        }
        Err(_) => {
            windows_console::println_japanese("設定ファイルが見つかりません。サンプルconfig.tomlを作成しています...");
            Config::create_sample_file(config_path)?;
            Err("サンプルconfig.tomlを作成しました。Zoom API認証情報を設定して再実行してください。".into())
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
        .add_scope(Scope::new("user:read".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    windows_console::println_japanese("アプリケーションを認証するために以下のURLにアクセスしてください:");
    windows_console::println_japanese(&auth_url.to_string());
    
    #[cfg(windows)]
    {
        windows_console::println_japanese("デフォルトブラウザでURLを開いています...");
        windows_console::println_japanese("ブラウザが自動で開かない場合は、上記URLを手動でコピーして貼り付けてください。");
        open_url_windows(&auth_url.to_string());
    }
    
    windows_console::println_japanese("\n認証後、URLから認証コードをコピーしてください:");
    
    let mut auth_code = String::new();
    io::stdin().read_line(&mut auth_code)?;
    let auth_code = auth_code.trim();

    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    let access_token = token_result.access_token().secret();
    
    windows_console::println_japanese("アクセストークンを取得しました! 環境変数として設定できます:");
    if cfg!(windows) {
        windows_console::println_japanese(&format!("set ZOOM_ACCESS_TOKEN={}", access_token));
    } else {
        windows_console::println_japanese(&format!("export ZOOM_ACCESS_TOKEN={}", access_token));
    }
    
    Ok(access_token.to_string())
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
    // Windows では URL にアンパサンド(&)が含まれるとコマンドが分割されるため、
    // 引用符で囲んで実行する
    let _ = Command::new("powershell")
        .args(&["-Command", &format!("Start-Process '{}'", url)])
        .spawn()
        .or_else(|_| {
            // PowerShell が失敗した場合は cmd を使用
            Command::new("cmd")
                .args(&["/C", "start", "\"\"", url])
                .spawn()
        });
}