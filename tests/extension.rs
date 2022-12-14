pub mod utils;

use eip5139::errors::*;
use eip5139::{RpcProviders, Source};

use futures_executor::LocalPool;

use self::utils::Fetch;

#[cfg(target_family = "wasm")]
use wasm_bindgen_test::wasm_bindgen_test;

#[test]
#[cfg_attr(target_family = "wasm", wasm_bindgen_test)]
fn valid_empty_add_one() {
    let one = r#"{
  "name": "Root List",
  "version": {
    "major": 0,
    "minor": 1,
    "patch": 1,
    "build": "XPSr.p.I.g.l"
  },
  "timestamp": "2004-08-08T00:00:00.0Z",
  "logo": "https://mylist.invalid/logo.png",
  "providers": {
  }
}"#;

    let two = r#"{
  "name": "Extension List",
  "version": {
    "major": 10,
    "minor": 1,
    "patch": 0,
    "build": "wWw"
  },
  "timestamp": "2024-08-08T00:00:00.0Z",
  "logo": "https://mylist2.invalid/logo.png",
  "extends": {
    "uri": "file://one",
    "version": {
      "major": 0,
      "minor": 1,
      "patch": 0
    }
  },
  "changes": [
    {
        "op": "add",
        "path": "/some-key",
        "value": {
            "name": "Frustrata",
            "chains": [
                {
                    "chainId": 1,
                    "endpoints": [
                        "https://mainnet1.frustrata.invalid/",
                        "https://mainnet2.frustrana.invalid/"
                    ]
                },
                {
                    "chainId": 3,
                    "endpoints": [
                        "https://ropsten.frustrana.invalid/"
                    ]
                }
            ]
        }
    },
    {
        "op": "add",
        "path": "/other-key",
        "value": {
            "name": "Sourceri",
            "priority": 3,
            "chains": [
                {
                    "chainId": 1,
                    "endpoints": [
                        "https://mainnet.sourceri.invalid/"
                    ]
                },
                {
                    "chainId": 42,
                    "endpoints": [
                        "https://kovan.sourceri.invalid"
                    ]
                }
            ]
        }
    }
]
}"#;

    let fetch = Fetch::with_two(one, two);
    let mut pool = LocalPool::new();
    let list = pool
        .run_until(RpcProviders::fetch(fetch, Source::Uri("file://two".into())))
        .unwrap();

    assert_eq!(list.name, "Extension List");
    assert_eq!(list.logo, Some("https://mylist2.invalid/logo.png".into()));
    assert_eq!(list.timestamp, "2024-08-08T00:00:00.0Z");

    assert_eq!(list.version().major, 10);
    assert_eq!(list.version().minor, 1);
    assert_eq!(list.version().patch, 0);
    assert_eq!(list.version().build, Some("wWw".into()));
    assert_eq!(list.version().pre_release, None);

    let mut providers = list.providers().to_vec();
    providers.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(providers.len(), 2);

    assert_eq!(providers[0].name, "Frustrata");
    assert_eq!(providers[0].logo, None);
    assert_eq!(providers[0].priority, None);

    let chains = &providers[0].chains;
    assert_eq!(chains.len(), 2);

    assert_eq!(chains[0].chain_id, 1);
    assert_eq!(
        chains[0].endpoints,
        [
            "https://mainnet1.frustrata.invalid/",
            "https://mainnet2.frustrana.invalid/"
        ]
    );

    assert_eq!(chains[1].chain_id, 3);
    assert_eq!(chains[1].endpoints, ["https://ropsten.frustrana.invalid/"]);

    assert_eq!(providers[1].name, "Sourceri");
    assert_eq!(providers[1].logo, None);
    assert_eq!(providers[1].priority, Some(3));

    let chains = &providers[1].chains;
    assert_eq!(chains.len(), 2);

    assert_eq!(chains[0].chain_id, 1);
    assert_eq!(chains[0].endpoints, ["https://mainnet.sourceri.invalid/",]);

    assert_eq!(chains[1].chain_id, 42);
    assert_eq!(chains[1].endpoints, ["https://kovan.sourceri.invalid"]);
}

#[test]
#[cfg_attr(target_family = "wasm", wasm_bindgen_test)]
fn invalid_version() {
    let one = r#"{
  "name": "Root List",
  "version": {
    "major": 0,
    "minor": 1,
    "patch": 1,
    "build": "XPSr.p.I.g.l"
  },
  "timestamp": "2004-08-08T00:00:00.0Z",
  "logo": "https://mylist.invalid/logo.png",
  "providers": {
  }
}"#;

    let two = r#"{
  "name": "Extension List",
  "version": {
    "major": 10,
    "minor": 1,
    "patch": 0,
    "build": "wWw"
  },
  "timestamp": "2024-08-08T00:00:00.0Z",
  "logo": "https://mylist2.invalid/logo.png",
  "extends": {
    "uri": "file://one",
    "version": {
      "major": 0,
      "minor": 2,
      "patch": 0
    }
  },
  "changes": []
}"#;

    let fetch = Fetch::with_two(one, two);
    let mut pool = LocalPool::new();
    let err = pool
        .run_until(RpcProviders::fetch(fetch, Source::Uri("file://two".into())))
        .unwrap_err();

    match err {
        Error::VersionMismatch { .. } => (),
        other => panic!("expected VersionMismatch, but got: {:?}", other),
    }
}
