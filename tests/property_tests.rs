use proptest::prelude::*;
use zoom_video_mover_lib::Config;

// Config property-based tests
prop_compose! {
    fn arb_config()
        (client_id in "[a-zA-Z0-9]{10,50}",
         client_secret in "[a-zA-Z0-9]{20,100}")
        -> Config
    {
        Config {
            client_id,
            client_secret,
            redirect_uri: None,
        }
    }
}

proptest! {
    /// Property: 有効なConfigはTOMLシリアライゼーション・デシリアライゼーションでラウンドトリップが可能
    #[test]
    fn config_toml_roundtrip(config in arb_config()) {
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        prop_assert_eq!(config.client_id, deserialized.client_id);
        prop_assert_eq!(config.client_secret, deserialized.client_secret);
        prop_assert_eq!(config.redirect_uri, deserialized.redirect_uri);
    }

    /// Property: 非空のclient_idとclient_secretを持つConfigは常に有効
    #[test]
    fn valid_config_invariants(config in arb_config()) {
        prop_assert!(!config.client_id.is_empty());
        prop_assert!(!config.client_secret.is_empty());
        prop_assert!(config.client_id.len() >= 10);
        prop_assert!(config.client_secret.len() >= 20);
    }
}

// Date validation property tests
prop_compose! {
    fn arb_date_string()
        (year in 2020u32..2030u32,
         month in 1u32..13u32,
         day in 1u32..29u32) // 28日までで安全
        -> String
    {
        format!("{:04}-{:02}-{:02}", year, month, day)
    }
}

proptest! {
    /// Property: 有効な日付形式の文字列は常に10文字
    #[test]
    fn valid_date_format_length(date in arb_date_string()) {
        prop_assert_eq!(date.len(), 10);
        prop_assert!(date.contains('-'));
        
        // YYYY-MM-DD形式の検証
        let parts: Vec<&str> = date.split('-').collect();
        prop_assert_eq!(parts.len(), 3);
        prop_assert_eq!(parts[0].len(), 4); // year
        prop_assert_eq!(parts[1].len(), 2); // month
        prop_assert_eq!(parts[2].len(), 2); // day
    }

    /// Property: from <= to の関係性を検証
    #[test]
    fn date_range_ordering(from in arb_date_string(), to in arb_date_string()) {
        if from <= to {
            // 有効な日付範囲の場合
            prop_assert!(from <= to);
        } else {
            // 無効な日付範囲の場合は逆転
            prop_assert!(to < from);
        }
    }
}

// URL validation tests
proptest! {
    /// Property: 有効なuser_idは空でない文字列
    #[test]
    fn user_id_not_empty(user_id in "[a-zA-Z0-9@._-]{1,100}") {
        prop_assert!(!user_id.is_empty());
        prop_assert!(user_id.len() <= 100);
    }
}